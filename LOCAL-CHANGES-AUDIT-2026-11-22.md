# Audit des changements locaux — 22 novembre 2026

## Résultat

Audit transversal des différences Git, des points d'intégration et des contrôles des ensembles concernés. Ce n'est pas une certification ligne par ligne de tout le fork.

- Moteur : `D:/Github/Projets/IA/deepseek-harness`, branche `fix/lm-studio-devstral-roles`, HEAD `7f45dce5dfb0e17231b39da706e53ed061b18c4e`.
- Lanceur : `D:/Github/dandy-harness`, branche `main`, HEAD `cc607829b7584ae1b870f19770de44a6f0477f2a`.
- Après `git fetch origin`, les deux HEAD correspondent exactement à leur branche distante : zéro commit en avance ou en retard. Le HEAD moteur n'est pas ancêtre de `origin/master` : publié sur la branche ne veut pas dire fusionné dans master.
- Au départ : 64 fichiers suivis modifiés côté moteur, plus les fichiers non suivis ; aucun code modifié côté lanceur, seulement cinq rapports et un raccourci non suivis.
- Aucun commit, push, changement de fournisseur/modèle, chargement de LLM ou redémarrage de Harness effectué pendant cet audit. Inkling reste inchangé. Les artefacts du moteur ont été reconstruits, mais cela ne remplace pas les modules déjà chargés dans le processus ouvert.

## Ce qui est intégré, testé et publié

« Intégré » désigne ici le branchement dans le code ou le build ; cela ne certifie pas l'activation de chaque plugin dans tous les profils utilisateur.

| Ensemble | Intégration observée | Publication | Vérification actuelle |
|---|---|---|---|
| Fenêtre Tauri, supervision Node, Job Object, DLL de processus embarquée | Code et build du lanceur ; compilation du helper par `build.rs` | Dans `origin/main` ; helper moteur dans le commit `6ac4a2798e` | 8 tests Rust réussis, 1 benchmark ignoré ; arrêt réel de descendants testé ; Clippy strict réussi |
| Démarrage concurrent et réduction mémoire WebView2 minimisé | Code du lanceur | Dans `origin/main` (`3f29d7b`, `d581c48`) | Compilation/tests Rust ; pas de nouvelle mesure mémoire ni de nouveau parcours visuel dans cet audit |
| Recherche de fichiers | Outils TS déléguant à ripgrep natif/Rust | Fichiers suivis sans différence locale | 181 tests réussis, 1 ignoré |
| Optimisations app-boot, résolution des paquets et HMR | Sources locales et artefacts reconstruits | Différences non commitées | Suites profil/résolution/HMR dans le lot de 356 réussites ; un cas de lien symbolique bloqué par Windows |
| Chargement différé HTML/Markdown du web fetch | Source locale `fetch.ts` | Non commité | 109 tests web réussis |
| Consignes globales et éditeur | API, contrôle propriétaire liveshare et interface connectés ; nouveaux fichiers requis | Non commité, certains fichiers non suivis | Tests API et composant exécutés, compilation Host/Client réussie |
| Contexte LM Studio, compactage et outils à la demande | Adaptateur, boucle agent, service outils et exécuteur connectés | Différences non commitées | Inclus dans les 835 tests ciblés réussis ; tests de catalogue, budget et exécuteur inclus |
| Rappels de répétition et récupération DGX | Rappel branché ; récupération conditionnelle au fournisseur et au fichier local de configuration | Non commité | Tests simulés exécutés ; aucune connexion réelle DGX tentée |
| Missions multi-LLM et validation du travail | Paquets expérimentaux, compilés, chargés explicitement par les tests de composition | Répertoires non suivis, donc absents du dépôt distant | Tests unitaires et Loader réussis dans les 835 ; essais fournisseur ignorés sans configuration. Pas d'activation automatique effectuée |
| Historique/budget Rust WASM | Pont optionnel dans le scanner Zstd et le calcul de seuil | Sources/manifestes locaux, crate non suivi | 2 tests Rust, 254 cas différentiels, test ABI ; reste désactivé par défaut, aucun gain justifiant l'activation |
| Client MCP différé | Expérience retirée au tour précédent | Aucune différence locale dans `packages/mcp` | Rapport MCP local ; pas de nouvelle tentative ni activation |
| Archivage des sous-agents | Fonctionnalité déjà suivie ; correction de validation ajoutée pendant cet audit | La correction présente n'est pas publiée | 506 tests sous-agents réussis, 1 ignoré, dont archivage/restauration et nouveau contrôle du checkpoint |

Les rapports d'historiques, d'audit Rust et des expériences plugins/MCP restent non suivis côté lanceur. Les journaux et corpus d'évaluation sous `packages/experimental/work-validation/evaluation` ne sont pas à publier aveuglément : conserver seulement les éléments reproductibles appropriés après revue de leur contenu. Le dossier `.repolens` et le raccourci Windows ne constituent pas du code à livrer.

## Stabilisation réalisée

1. `native/system/packages/entry/src/kernels.ts` : suppression de l'assertion `as unknown`; validation de la mémoire et de tous les exports WASM requis avant vérification de version et mise en cache. Un artefact incomplet échoue avec une erreur ABI explicite.
2. `native/system/test/kernels.test.js` : module WASM synthétique valide, retrait successif de chacun des exports, version incompatible et chemin non activé. Aucun compilateur Rust nécessaire à ce test de frontière.
3. `packages/subagent/subagent/src/popup-archive.ts` : remplacement des deux assertions `as unknown` par des validateurs typés conservant les contraintes chaîne et entier non négatif. Format de données inchangé.
4. `packages/subagent/subagent/tests/popup-archive.spec.ts` : validation d'un checkpoint correct et refus des identifiants/offsets invalides. Le test existant d'archivage/restauration reste passant.
5. README du kernel : commande du nouveau test et contrat ABI complété.

Le garde-fou `verify-no-unknown-casts` passe désormais, avec les 1 214 assertions historiques admises par sa référence existante. Aucune référence du garde-fou n'a été relâchée.

## Preuves exécutées

Toutes les commandes ont été exécutées directement pendant cet audit, sans sous-agent.

| Contrôle | Résultat |
|---|---|
| Sélection adaptateur, compaction, agent-loop/exécuteur, settings API, répétitions, retry, missions, work-validation | 51 fichiers réussis, 3 ignorés ; 835 tests réussis, 4 ignorés |
| Profils app-boot, résolution, HMR, Zstd et composant settings | 4 fichiers réussis, 1 échoué ; 356 tests réussis, 1 échec `EPERM` de création d'un lien symbolique |
| Web tools | 4 fichiers, 109 tests réussis |
| Sous-agents après correction | 27 fichiers, 506 tests réussis, 1 ignoré |
| Recherche réelle avec ripgrep | 5 fichiers, 181 tests réussis, 1 ignoré |
| Kernel ABI Node | 1 test réussi couvrant 7 modules incomplets/incompatibles |
| Kernel Rust natif | 2 tests réussis |
| Kernel différentiel JS/WASM | 254 cas réussis ; expérience toujours désactivée |
| Lanceur Rust MSVC | 8 tests réussis, 1 benchmark ignoré |
| Lanceur Clippy `-D warnings` | Réussi |
| Helper rust-process `cargo test` | Compilation réussie mais **zéro test** dans cette crate ; ce résultat seul ne prouve pas le nettoyage |
| Dépendances publiées | 101 paquets conformes à la politique vérifiée |
| Build final moteur | `build:artifacts` : 8 contrôles réussis ; puis `build:check` : 2 contrôles réussis |
| Diff Git | `git diff --check` réussi |

La première sélection large de tests Node n'a pas terminé et a été interrompue après un délai anormal. Son résultat global n'est pas déclaré réussi. Les ensembles utiles ont ensuite été rejoués séparément avec les résultats ci-dessus. Cette première exécution signalait également des échecs de fixtures de liens symboliques dans package-meta, compatibility-preflight, génération JSONL et package-reload. Un cas a été reproduit seul : `EPERM` sur `symlinkSync` avant entrée dans le code testé. Les privilèges Windows n'ont pas été changés et ces tests n'ont pas été neutralisés. Une validation sur un environnement autorisant ces liens reste nécessaire.

Le shell ne trouvait initialement ni Cargo ni le linker GNU. La commande correcte utilise le Cargo installé et la chaîne **MSVC**, qui a réussi sans installation ni modification de la chaîne par défaut. `gh` est absent du PATH : aucune affirmation nouvelle sur l'état de la PR ou une release installable ; la publication des commits est établie par Git et les références récupérées.

## Reproduction et journaux

Journaux bruts : `target/local-audit-2026-11-22/` côté lanceur. L'inventaire exact des fichiers du moteur est dans `engine-status.txt`, avec `engine-diff-stat.txt`. Ces sorties locales ne sont pas un manifeste de fichiers à commiter.

```powershell
# Moteur, depuis son dépôt
pnpm exec vitest run packages/subagent/subagent/tests
pnpm --dir native/system/packages/entry run build:js
node --test native/system/test/kernels.test.js
node --import tsx/esm native/system/rust-kernels/verify.mjs
pnpm exec tsx scripts/verify-no-unknown-casts.ts
pnpm exec tsx scripts/verify-package-dependencies.ts
pnpm run build:artifacts
pnpm run build:check

# Lanceur, depuis son dépôt
& "$env:USERPROFILE\.cargo\bin\cargo.exe" +stable-x86_64-pc-windows-msvc test --locked --target x86_64-pc-windows-msvc
& "$env:USERPROFILE\.cargo\bin\cargo.exe" +stable-x86_64-pc-windows-msvc clippy --locked --target x86_64-pc-windows-msvc -- -D warnings
```

## Limites et prochains lots

- Ne pas annoncer l'ensemble du fork entièrement stabilisé : les fixtures Windows ci-dessus, la suite exhaustive, les gates documentaires/persistance et la matrice Linux/macOS ne sont pas validées par cet audit.
- Aucun nouvel essai LLM, Tauri visuel ou test distant MCP ici. Le test réel Inkling du tour précédent reste une preuve antérieure distincte, pas un retest des modifications présentes.
- Les fonctionnalités locales doivent être publiées par ensembles cohérents avec leurs nouveaux fichiers, manifestes et tests ; pousser seulement les fichiers suivis laisserait notamment les modules expérimentaux et le pont kernel absents.
- Les kernels WASM ne doivent pas être activés pour « finir le portage » : le pont reste plus lent dans le contrôle différentiel local. La recherche ripgrep et la gestion des processus Rust sont déjà intégrées.
- La distribution Tauri reste dépendante du checkout moteur construit, comme l'indique le README. Un installateur autonome n'a pas été créé par cet audit.
- Aucune dépense fournisseur, modification des données utilisateur ni changement d'Inkling effectué.
