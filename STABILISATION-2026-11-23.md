# Stabilisation et livraison locale — 23 novembre 2026

## Résultat livré

Installateur NSIS x64 par utilisateur, avec Node 24.22.3 et sa licence embarqués.
L'installation silencieuse a retourné 0. L'exécutable installé dans
`%LOCALAPPDATA%/Dandy Harness` a ouvert le chat existant et lancé son propre
`runtime/node.exe`. Glimmer et Auto review sont restés sélectionnés.
Le moteur dépend encore du checkout construit : le paquet n'est pas autonome.

Installateur : `target/installer/x86_64-pc-windows-msvc/release/bundle/nsis/Dandy Harness_0.1.0_x64-setup.exe`.
SHA-256 : `42E02F312FE11FE7A8673229CE12C44B88CB90F16505EC7D912393F07949F178`.

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
Les cinq échecs de liens symboliques restent visibles, sans désactivation de tests.

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
