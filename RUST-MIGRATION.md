# Portage Rust — inventaire et livraison progressive

État inspecté le 20 novembre 2026. Ce document décrit des frontières observées,
pas un pourcentage de portage calculé au nombre de fichiers.

## Architecture constatée

`src/main.rs` héberge Tauri/WebView2 et lance le profil `dsh web` du dépôt moteur.
L’interface et le moteur restent ceux de DeepSeek Harness. Cordis compose les
services, événements et plugins TypeScript. Les expressions de configuration
`!!js` sont également du JavaScript. `native/system` fournit des primitives
natives Landlock/flock ; cela ne signifie pas que le moteur est déjà en Rust.
La décompression Zstandard de Node est déjà native.

Sources inspectées dans `D:/Github/Projets/IA/deepseek-harness` :
`AGENTS.md`, `docs/architecture.md`, `native/system/README.md`,
`packages/boot/app-boot/src/index.ts`, `packages/llm/llm-pi-ai/src/adapter.ts`,
`packages/core/tool-executor-native/src/index.ts`,
`packages/session/session-persistence-jsonl/src/{format,zstd}.ts`.

## Inventaire par frontière

| Domaine | État actuel | Cible Rust et contraintes | Ordre |
|---|---|---|---|
| Fenêtre, lancement, supervision | Tauri Rust, lanceur minimal | Propriétaire du processus, délai de démarrage, erreurs et sortie bornée | 1 — implémenté dans ce lot |
| Fichiers, recherche, surveillance | Plugins TS et outils système | Service natif derrière les mêmes permissions, annulation et résultats d’outils ; réutiliser les exécutables natifs existants | 2 |
| Commandes et terminaux | Groupes shell/subprocess/terminal/sandbox | Processus, flux et arrêt d’arbre natifs ; les validations d’accès doivent rester obligatoires | 2 |
| Catalogue et transport LLM | Adaptateur TS pi-ai | HTTP/SSE, délais et annulation ; conserver les formats fournisseurs, credentials, outils et instantanés de sélection | 3 |
| Budget de contexte et outils à la demande | TS, dépend du modèle résolu | Calculs purs avec tests différentiels ; ne pas confondre estimation et tokenizer réel | 3 |
| Sessions et stockage | JSONL Zstd, validations/migrations TS | Lecteur natif puis écriture transactionnelle ; compatibilité stricte, verrous, fichiers interrompus et événements inconnus | 4 |
| Compactage | Plugins TS et requêtes LLM | Politique orchestrée en Rust ; préserver dernière consigne, résultats d’outils et place pour la réponse | 4 |
| Agents, sous-agents, missions | Boucle et événements Cordis | Machine d’état Rust ; vérifier annulation, reprise, budgets, délégations et absence de doublons | 5 |
| Paramètres et API locale | Services/plugins TS | API native compatible avec le frontend ; authentification locale inchangée | 5 |
| Chargeur de plugins | Cordis, imports TS, HMR, expressions JS | Refonte ou pont JS explicite. Les plugins existants ne deviennent pas des plugins Rust automatiquement | 6 |
| Interface | React/TypeScript dans WebView2 | À conserver : indépendante du langage du moteur. UI native Rust = autre réécriture | Hors migration moteur |

Les lignes 2 à 6 sont un inventaire de migration, pas des fonctionnalités livrées.
Leur découpage précis exige de lire leurs contrats et tests avant modification.
Un moteur entièrement Rust est possible mais ne peut conserver les extensions JS
inchangées sans un runtime JS ou un processus de compatibilité.

## Lot 1 : supervision Rust

`src/engine.rs` possède le processus Node, lit sa sortie avec une ligne limitée à
16 Kio, accepte seulement l’annonce de disponibilité de l’origine locale attendue,
attend au maximum 60 secondes au démarrage et détecte un arrêt. La fenêtre est
créée avant de tenter le lancement de Node : un exécutable absent produit une
erreur visible au lieu d’une sortie silencieuse. Le titre signale aussi l’arrêt
d’un moteur prêt, sans remplacer la conversation.

La limite de 60 secondes concerne **uniquement le démarrage**, jamais une
génération ou un appel d’outil. Pas de redémarrage automatique : aucun outil
potentiellement non idempotent n’est rejoué. Pas de logs ni de jetons d’accès
persistés par ce superviseur. Les lignes trop longues sont drainées et ignorées.
L’arrêt tue et attend le processus Node possédé. Sous Windows, un Job Object
possède désormais Node et ses descendants ; sa fermeture termine les processus
du groupe, y compris si Tauri disparaît. Node est créé suspendu, associé au Job,
puis repris : aucun descendant ne peut être créé avant cette association.
Une erreur d’association ou de reprise fait échouer le lancement et nettoie Node.
Les autorisations sandbox et les mécanismes d’annulation par outil restent en place.

Ce lot renforce une frontière Rust existante ; il ne porte pas encore la boucle
agent TypeScript et ne supprime pas Node. Le déplacement de cette boucle vient
après les contrats des services et la stratégie de compatibilité des plugins.

## Critères et vérification

- Exécutable absent : erreur retournée ; plus d’échec avant création de fenêtre.
- Moteur muet : délai borné, processus arrêté et attendu.
- Moteur terminé : détection avant échéance.
- Annonce valide : même URL transmise à Tauri, y compris son authentification.
- Origine différente : jamais acceptée.
- Sortie volumineuse : ligne bornée, prochaine ligne valide toujours reconnue.
- Arrêt répété : sans erreur ; aucun changement aux sessions ou fournisseurs.

```powershell
cargo +stable-x86_64-pc-windows-msvc test --locked --target x86_64-pc-windows-msvc
cargo +stable-x86_64-pc-windows-msvc build --locked --target x86_64-pc-windows-msvc
```

Les tests utilisent de vrais petits processus Node locaux simulant le protocole
de disponibilité. Ils ne constituent pas un essai de génération LLM ou un test
visuel Tauri. Aucun chargement de modèle n’est nécessaire.

## Mesures et règle de décision

### Lot du 20 novembre : résultats et activation

- Processus : nettoyage Windows des descendants implémenté dans `src/process_job.rs`.
  Test réel Node → enfant Node, arrêt du moteur, attente du handle enfant : réussi.
- Fichiers : recherche et énumération déjà déléguées à ripgrep (Rust) dans
  `tool-fs-search` ; 181 tests réussis, un ignoré. RepoLens reste complémentaire pour
  les symboles/dépendances ; aucune réécriture de son index n’est ajoutée.
- Historiques : index structurel des frames porté en Rust/WASM, branché en option
  dans `scanZstdFrames`. Compression/décompression Zstd déjà natives conservées.
- Budget : seuil pur porté dans le même module, tests différentiels ; pas un tokenizer.
- Activation : les kernels restent **désactivés par défaut**. Sur 20 paires alternées,
  index TS 0,01347 ms contre Rust 0,01718 ms ; calcul de seuil TS 0,00000333 ms contre
  Rust 0,00037221 ms. Le pont coûte davantage que ces calculs. Aucun gain de démarrage
  ou de mémoire totale n’est démontré pour ces deux ports.
- Reproduction et limites : `native/system/rust-kernels/README.md` dans le dépôt moteur.
  La restauration JSON et les requêtes de sessions restent en TypeScript.

Le profil précédent de la plus grosse session locale (287 événements, 676 889
octets décompressés, 30 essais après 5 chauffes, Node 24.22.3) donne une médiane de
3,98 ms : lecture 0,64 ms, décodage/copies 1,13 ms, restauration 2,14 ms.
Cela ne mesure ni le boot complet ni l’interface. Réécrire ce lecteur en priorité
ne supprimerait donc pas les secondes d’initialisation des plugins.

Pour chaque futur lot : même fixture, alternance ancien/nouveau chemin, contrôle
d’équivalence, temps médian/p95, mémoire du processus et limites annoncées.
Ne pas attribuer au Rust une accélération du LLM ou un gain non mesuré.

Comparatif exécuté du lecteur de logs, compilation Rust optimisée, même entrée
synthétique (ligne de 8 Mio suivie d’une annonce), 4 chauffes puis 20 paires en
ordre alterné : ancien `BufRead::lines` **8,02 ms**, lecteur borné **1,34 ms**
en médiane. Capacité du tampon de ligne : **16 Mio → 16 Kio**. Cela mesure un
cas de sortie anormalement longue, pas la mémoire totale de Tauri ni le démarrage
habituel. Le gain vient du bornage et de l’absence de copie des lignes rejetées,
pas d’un changement de langage : les deux lecteurs sont en Rust.

Reproduction sans reconstruire les dépendances Tauri :

```powershell
rustc +stable-x86_64-pc-windows-msvc --edition=2021 --test -O src/engine.rs -o target/supervisor-tests.exe
.\target\supervisor-tests.exe
.\target\supervisor-tests.exe --ignored --nocapture
```
