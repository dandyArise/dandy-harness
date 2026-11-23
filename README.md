# Dandy Harness

Prototype Windows de Harness avec une fenêtre Tauri 2 et WebView2. Le processus Rust démarre le moteur Node du fork local sur le port 18643 et ouvre l’interface web du fork. Les sessions et le profil web du prototype se trouvent dans `data/`, exclu de Git.

## Lancer

Depuis PowerShell dans ce dossier, avec Node dans le PATH :

```powershell
.\target\x86_64-pc-windows-msvc\debug\dandy-harness.exe
```

Le lanceur cherche Node dans cet ordre : `DANDY_NODE_EXECUTABLE` si défini,
`runtime/node.exe` à côté du lanceur, puis le `PATH`. Cela permet de fournir
un runtime avec la distribution sans dépendre du terminal qui ouvre l'application.

Le moteur utilise par défaut le fork construit dans `D:\Github\Projets\IA\deepseek-harness`. Définir `DANDY_HARNESS_SOURCE` pour employer un autre checkout qui dispose de ses dépendances et artefacts web. Le profil LM Studio importé lit `LM_API_TOKEN` dans l’environnement du processus. Une fois ouvert, utiliser la barre latérale pour sélectionner une conversation.

## Construire

### Installateur Windows local

Avec pnpm, Rust MSVC et les outils C++ Visual Studio disponibles :

```powershell
.\build-installer.ps1
```

Le script vérifie le SHA-256 officiel de Node 24, embarque son exécutable et sa
licence, puis produit `target/installer/x86_64-pc-windows-msvc/release/bundle/nsis/Dandy Harness_0.1.0_x64-setup.exe`.
L'installation est par utilisateur, dans `%LOCALAPPDATA%/Dandy Harness`.
Le runtime Node est inclus ; le moteur reste fourni par le checkout décrit
ci-dessus. Ce paquet n'est donc pas encore une distribution autonome du moteur.

Le lanceur Rust utilise `apps/cli/lib/bin.js` quand la CLI est compilée. Après une modification des sources de la CLI, reconstruire cet artefact pour que le prochain lancement utilise la modification. Si le fichier compilé est absent, le lanceur charge `apps/cli/src/bin.ts` via `tsx`.

Dans un terminal développeur Visual Studio x64 avec Rust MSVC :

```powershell
cargo +stable-x86_64-pc-windows-msvc build --locked --target x86_64-pc-windows-msvc
```

Le prototype dépend du checkout du moteur et du dossier de projet pour ses données. Les intégrations Electron spécifiques et la distribution autonome restent à porter. L’icône provient du fork Harness, dont la licence accompagne le fichier dans `icons/`.

## Supervision et portage Rust

Sur Windows, `cargo build` compile automatiquement le helper du moteur
`native/system/rust-process` en release, pour la même cible que Tauri, puis
incorpore sa DLL dans l’exécutable. Le checkout indiqué par `DANDY_HARNESS_SOURCE`
(ou celui par défaut) doit donc contenir ses sources et son lockfile.
Au lancement, la DLL est extraite dans `data/native/<empreinte>/dandy_process.dll`
et transmise au moteur. Aucune copie manuelle depuis `target/release` n’est nécessaire.
Une DLL différente utilise un autre dossier pour ne pas remplacer un module chargé.
Un fichier existant différent du contenu embarqué provoque une erreur explicite.
Une valeur explicite de `DANDY_WINDOWS_PROCESS_DLL` reste prioritaire.
Le build échoue si le helper ne compile pas ; il ne livre pas un repli silencieux.

Le moteur dispose d’un délai de démarrage de 60 secondes (sans limite ajoutée aux
générations). En cas d’échec, la fenêtre reste ouverte et son titre indique la
cause. Un arrêt après démarrage est également signalé, sans recharger la conversation.
Le superviseur ne redémarre pas automatiquement les tâches.

Node démarre en parallèle de la création de WebView2 ; une erreur de lancement
reste affichée dans la fenêtre. Le handle supervisé est conservé dès le lancement,
y compris si la création de la fenêtre échoue.

Le profilage est désactivé par défaut. `DANDY_STARTUP_PROBE` désigne un fichier
local de jalons (millisecondes depuis l’entrée Rust, sans contenu de conversation).
Dans ce mode uniquement, `DANDY_STARTUP_SERIAL=1` rétablit le démarrage séquentiel
pour comparaison. `DANDY_NODE_PROFILE` permet un profil CPU Node de 4,5 secondes
dans le fichier indiqué ; ce diagnostic contient des chemins de modules locaux.
Voir [le comparatif démarrage et mémoire](STARTUP-PERFORMANCE-2026-11-20.md).

`DANDY_NODE_MEMORY=compact` active une variante expérimentale : zone temporaire
V8 de 4 Mio pour le moteur Node seulement. Ce n’est pas une limite du tas total.
La variable absente conserve le comportement normal. Voir les mesures et limites
dans [le rapport du 23 novembre](MEMORY-STARTUP-2026-11-23.md).

[Inventaire, frontières et validations du portage Rust](RUST-MIGRATION.md).
