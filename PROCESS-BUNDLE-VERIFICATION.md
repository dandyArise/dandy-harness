# DLL processus embarquée — vérification du 20 novembre 2026

## Livraison locale

- `build.rs` compile le helper du checkout moteur avec Cargo `--locked --release`, pour la cible Tauri, dans un target isolé sous OUT_DIR.
- La DLL est incorporée dans l’EXE puis extraite sous `data/native/<empreinte>/dandy_process.dll`. Une configuration explicite `DANDY_WINDOWS_PROCESS_DLL` reste prioritaire.
- EXE vérifié : `target/x86_64-pc-windows-msvc/debug/dandy-harness.exe`.
- Le moteur Node et son checkout restent des dépendances : ce n’est pas encore une distribution autonome.

## Contrôles exécutés

- `cargo +stable-x86_64-pc-windows-msvc test --locked --target x86_64-pc-windows-msvc` : 8 réussis, 1 benchmark ignoré. Inclut extraction/réutilisation/version/corruption, arrêt du moteur et descendants.
- `cargo +stable-x86_64-pc-windows-msvc build --locked --target x86_64-pc-windows-msvc` : réussi. Aucune copie manuelle de DLL.
- Clippy avec `-D warnings` : réussi après remplacement du `loop`/`let … else` par `while let` dans le lecteur de logs. Les 8 tests ont été réexécutés avec succès après cette correction.

## Parcours réel Tauri

EXE PID 25636, moteur Node PID 1892. Session « Test d’arrêt uniquement PowerShell », Glimmer v7 conservé.

1. Le modèle lance le script temporaire tree.cjs en arrière-plan (job `pwsh-0c88d4251ce64999`).
2. Les trois Node de test 2736, 30472, 32004 sont observés vivants, avec lanceurs 30348 et 24768.
3. Clic Arrêter dans Tâches en arrière-plan. Les cinq PID ont disparu lors de deux contrôles Windows ; le chat rapporte l’arrêt demandé et les trois lignes READY.
4. Création d’un terminal via le panneau Tauri. Le moteur charge effectivement `data/native/4792ff35bdcc13c2/dandy_process.dll` (module Windows observé), et démarre cmd.exe 32084 / OpenConsole.exe 28148.
5. Fermeture de l’onglet terminal : les deux PID disparaissent. Harness et son moteur restent actifs.

Le test de commande exerce le confinement existant des commandes ; celui du terminal confirme le chargement et l’utilisation de l’intégration native. Aucun test de crash ni de descendant volontairement échappé n’a été effectué dans ce parcours. Aucun fichier de projet utilisateur n’a été modifié par le modèle.
