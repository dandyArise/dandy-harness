# Stabilisation et livraison locale — 23 novembre 2026

## Résultat livré

Installateur NSIS x64 par utilisateur, avec Node 24.22.3 et sa licence embarqués.
L'installation silencieuse a retourné 0. L'exécutable installé dans
`%LOCALAPPDATA%/Dandy Harness` a ouvert le chat existant et lancé son propre
`runtime/node.exe`. Glimmer et Auto review sont restés sélectionnés.
Cette première installation dépendait encore du checkout construit. La section
« Complément » ci-dessous décrit les corrections et les nouvelles vérifications.

Installateur : `target/installer/x86_64-pc-windows-msvc/release/bundle/nsis/Dandy Harness_0.1.0_x64-setup.exe`.
SHA-256 du nouvel installateur autonome : `1E379F0E38388E1EAA62BFD4BFB0DF0061EEE33F9311428C266E46BA769E9CC8`.

## Vérifications exécutées

- Lanceur : `cargo +stable-x86_64-pc-windows-msvc test --target x86_64-pc-windows-msvc` : 10 réussis, 1 ignoré.
- Clippy, tous les targets, avertissements interdits : réussite.
- Moteur : `pnpm run lint` et `pnpm run build` : réussite, 10 portes de build.
- Adaptateur, popup et workspace : 734 réussis, 1 échec, 1 ignoré initialement.
  L'échec était une assertion obsolète demandant de conserver les conversations
  après suppression du groupe. Correction selon le comportement demandé ;
  nouvelle exécution workspace + contrats CI : 198 réussis.
- Reprise, payloads Devstral, retry et archivage : 197 réussis.
- Audit des modules locaux : 1249 réussis, 2 échecs EPERM de liens symboliques,
  13 ignorés (dont essais fournisseurs non configurés).
- Contrôles supplémentaires workspace, budget et présentation : 115 réussis,
  3 échecs EPERM de liens symboliques Windows.
- Éditeur de consignes : 25 réussis. Noyaux Rust : 2 réussis.
- Documentation : `pnpm run doc-sync` : 43 portes réussies.

Ces groupes se recouvrent ; leurs nombres ne constituent pas un total unique.
Les cinq échecs de liens symboliques étaient présents lors de ce premier passage ;
ils sont résolus dans le complément ci-dessous, sans désactivation de tests.

## Limites du parcours réel

La relance du sous-agent de test a bien affiché En cours puis Échoué.
LM Studio a rejeté `muse-glimmer` avec `model_not_found`, avant génération.
Aucun modèle n'a été remplacé ou rechargé. Le chat réel, le premier outil et
l'archivage/restauration d'un agent nouvellement terminé restent à retester
quand cet identifiant sera disponible. Les tests automatisés ne remplacent
pas cette preuve Tauri. Aucune amélioration de performance nouvelle n'est revendiquée.

## Reproduction

Construire le moteur avec `pnpm run build`, puis exécuter `./build-installer.ps1`
dans ce dépôt avec pnpm et Rust MSVC disponibles. Le build utilise un répertoire
isolé pour ne pas écraser un exécutable déjà ouvert.

## Complément — liens Windows, migration Glimmer et moteur embarqué

- Après activation du Mode développeur par l'utilisateur : 308 tests de
  `profile-resolution.spec.ts` et `generation.spec.ts` réussis, y compris les
  liens symboliques vers des fichiers. Les trois liens de dossiers du workspace
  utilisent des jonctions sous Windows : 90 tests workspace réussis.
- Identifiant réellement disponible : `meta/muse-glimmer`. Le catalogue et le
  modèle par défaut utilisent cet identifiant ; Glimmer et Auto review sont
  conservés. Aucune clé, aucun chargement LM Studio et aucun fine-tuning modifiés.
- Régression reproduite : pendant un changement de modèle, le résumé essayait
  l'ancien identifiant supprimé avant l'enregistrement du nouvel en-tête.
  Le repli utilise maintenant la destination proposée, uniquement sur le même
  fournisseur et uniquement pour `UNKNOWN_MODEL`. Nettoyage du repli dans
  `finally` ; 183 tests de compactage réussis.
- Tauri, conversation existante « Test de lecture seule : » : réponse « Bonjour »
  en 13 s. Puis découverte de `read`, lecture réelle du README et réponse
  « Suivi dépenses - FastAPI + React » en 46 s, avec Auto review conservé.
- API réelle, test séparé : appel structuré d'un outil de test et restitution
  d'une valeur aléatoire réussis. Ce test API ne remplace pas le parcours Tauri.
- Lanceur Rust : 11 tests réussis, 1 benchmark ignoré ; Clippy sans avertissement.
- Build moteur local : 8 portes réussies. Runtime de production isolé : HTTP 200,
  page HTML servie, puis arrêt de son propre processus de test.

Le packaging prépare les archives locales et leurs dépendances de production,
préserve les patches du dépôt et intègre le moteur sous `engine/`. Le lanceur
cherche ce moteur dans le répertoire de ressources Tauri, sauf override explicite
`DANDY_HARNESS_SOURCE`. Les données existantes restent à leur emplacement ; sur
un nouveau poste elles vont dans `%LOCALAPPDATA%/Dandy Harness/data`.

Reproduction : `./build-installer.ps1` prépare et vérifie automatiquement le
moteur. Pour réutiliser un staging déjà vérifié :
`./build-installer.ps1 -EngineDirectory <staging> -Compression zlib`.
La compression est une option NSIS documentée par
[Tauri](https://v2.tauri.app/reference/config/#nsisconfig) ; LZMA reste la valeur
par défaut. La première tentative de packaging a rencontré un verrou Windows
après ouverture prématurée de l'exécutable ; ne pas lancer la sortie de build
avant la fin du bundling.

### Résultat final de l'installation autonome

- Build NSIS terminé avec code 0, installateur de 309 703 334 octets (295,36 MiB).
- Installation silencieuse `/S` terminée avec code 0.
- Lancement de `%LOCALAPPDATA%/Dandy Harness/dandy-harness.exe` depuis le dossier
  temporaire, sans `DANDY_HARNESS_SOURCE` ni `DANDY_NODE_EXECUTABLE`.
- Son processus moteur utilise le Node embarqué et l'entrée
  `node_modules/@deepseek-ai/dsh/lib/bin.js`. Zéro lien de réanalyse dans le
  répertoire moteur installé : aucune dépendance par jonction vers le checkout.
- Fenêtre Tauri installée observée : conversation et dernières réponses
  restaurées, éditeur disponible, Glimmer et Auto review conservés.
- Lint moteur réussi. Doc-sync : 42 portes réussies initialement, puis la porte
  de traduction corrigée et relancée sur les 1 303 paires, toutes cohérentes.

Les appels réels Glimmer ci-dessus ont été faits avec exactement le staging
embarqué dans ce nouvel installateur ; après installation, le lancement et la
restauration de l'interface ont été revérifiés. Aucun essai sur machine vierge
ou VM n'a été réalisé. Ces changements ont ensuite été commités et poussés.

RepoLens `gain . --format json` (cumuls historiques, estimations des plages de
lecture, pas une mesure propre à cette passe) : moteur 8 360 505 jetons estimés
économisés (89,9 %), lanceur 6 027 (28,4 %). Pas de repli sur `rg`.

## Livraison GitHub multiplateforme

Préversion : https://github.com/dandyArise/dandy-harness/releases/tag/v0.1.0

Le moteur est épinglé à `f0a26e7f33f5f69619c3793f61f2e4d6909ee3aa`.
Les paquets contiennent Node, sa licence et la fermeture des dépendances de
production du moteur. Aucun profil utilisateur ni clé API n'est embarqué.

| Plateforme | Exécution GitHub Actions | Résultat vérifié |
| --- | --- | --- |
| Windows x64 | 45203171703, job 137812474871 | 11 tests Rust réussis, 1 benchmark ignoré ; Clippy ; NSIS installé silencieusement ; moteur installé HTTP 200 |
| macOS Apple Silicon | 45214760172, job 137832869576 | 9 tests réussis, 1 benchmark ignoré ; Clippy ; DMG monté en lecture seule ; signature ad hoc vérifiée ; moteur du DMG HTTP 200 |
| Linux x64 | 45218262270, job 137845267489 | 9 tests réussis, 1 benchmark ignoré ; Clippy ; test du filtrage natif ; DEB et AppImage extraits ; moteur de chacun HTTP 200 |
| macOS Intel | 45214768025, job 137832900668 | Tests et Clippy réussis, compilation release réussie ; packaging échoué au démontage de l'image temporaire (`hdiutil: detach: timeout for DiskArbitration expired`) ; DMG non livré |

Windows et Apple Silicon utilisent respectivement les scripts des commits
`11bc820` et `b1e6e72`. Linux utilise `9201fc6`, qui filtre aussi le prébuild musl
de Koffi. Le diff de `src/`, `Cargo.toml`, `Cargo.lock`, `build.rs`,
`tauri.conf.json`, `assets/` et `icons/` entre ces révisions est vide : le code
applicatif est identique, les corrections suivantes concernent le packaging
et sa vérification. Le tag de la préversion pointe sur `b1e6e72`.

Les premières erreurs de packaging ont été corrigées : exécution des scripts
de packing avec le pnpm épinglé, icônes natives, variantes ELF musl/architectures
étrangères retirées uniquement du staging Linux, vérification du DMG livré au
lieu du dossier `.app` temporaire supprimé par Tauri. Le lanceur statique du
sandbox et les bibliothèques glibc sont conservés. Le moteur est retesté après
le filtrage, puis après extraction des paquets. Les jobs peuvent être relancés
par plateforme sans reconstruire celles déjà validées.

Les quatre installateurs disponibles et `SHA256SUMS.txt` ont été téléchargés ou
produits localement, publiés puis comparés aux digests SHA-256 retournés par
GitHub : concordance. Les noms des fichiers du manifeste correspondent aux
noms des assets téléchargés (espaces remplacés par des tirets).

Sur le poste existant, `%LOCALAPPDATA%/Dandy Harness/data` est une jonction vers
le dossier `data` de ce checkout. Cela conserve les conversations, Glimmer et
Auto review pour l'EXE compilé en CI, sans déplacer ni recopier les secrets.
Ne pas supprimer ce dossier de données historique tant que la jonction existe.

Limites : EXE sans certificat éditeur ; DMG signé ad hoc, sans notarisation
Apple ; parcours graphiques interactifs macOS et Linux non vérifiés.
La CI teste le moteur réellement contenu dans chaque paquet, ce qui ne remplace
pas un test visuel complet du chat sur ces systèmes. Aucun gain de performance
supplémentaire n'est revendiqué pour cette livraison.

Livraison disponible : Windows x64, macOS Apple Silicon et Linux x64. Le job Intel
est terminé en échec ; aucun travail GitHub Actions ne reste en attente pour cette
livraison. La variante Intel nécessite encore un correctif du packaging et un
nouveau contrôle du moteur extrait avant publication. Ne pas utiliser le DMG ARM
comme substitut sur un Mac Intel.
