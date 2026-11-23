# Dandy Harness

Application Harness avec une fenêtre Tauri 2. Le processus Rust démarre le moteur Node sur le port 18643 et ouvre son interface web. Les installateurs incluent Node et le moteur.

## Installer

Télécharger l'installateur correspondant à son système dans les
[releases GitHub](https://github.com/dandyArise/dandy-harness/releases).
Le workflow `Installers` construit Windows x64, macOS Apple Silicon/Intel et
Linux x64 (DEB et AppImage), vérifie les moteurs embarqués puis publie une
préversion avec `SHA256SUMS.txt`. [Instructions et limites de signature](RELEASE.md).
Chaque build utilise une révision précise du moteur indiquée dans le workflow.

## Lancer

Depuis PowerShell dans ce dossier, avec Node dans le PATH :

```powershell
.\target\x86_64-pc-windows-msvc\debug\dandy-harness.exe
```

Le lanceur cherche Node dans cet ordre : `DANDY_NODE_EXECUTABLE` si défini,
`runtime/node.exe` (ou `runtime/node`) dans les ressources Tauri, puis le `PATH`. Cela permet de fournir
un runtime avec la distribution sans dépendre du terminal qui ouvre l'application.

Le lanceur utilise le dossier `engine/` des ressources Tauri (à côté de l'exécutable Windows, dans `Contents/Resources` sur Mac). Pour un lancement de développement, définir `DANDY_HARNESS_SOURCE` vers le checkout compilé avant la commande ci-dessus. Un chemin explicite invalide produit une erreur, sans repli silencieux. Le profil LM Studio utilise la référence `LM_API_TOKEN` du gestionnaire de credentials ou de l'environnement.

`DSH_HOME` permet de choisir le dossier de données. Les installations existantes conservent leur ancien `data/` ; sur une nouvelle machine, les données vont dans `%LOCALAPPDATA%/Dandy Harness/data`. Les clés, sessions et paramètres personnels ne sont jamais inclus dans le paquet moteur.

Sur macOS : `~/Library/Application Support/Dandy Harness/data`. Sur Linux :
`$XDG_DATA_HOME/Dandy Harness/data`, ou `~/.local/share/Dandy Harness/data`.

## Construire

### Installateur Windows local

Avec pnpm, Rust MSVC et les outils C++ Visual Studio disponibles :

```powershell
.\build-installer.ps1
```

Le script vérifie le SHA-256 officiel de Node 24, embarque son exécutable et sa
licence, puis produit `target/installer/x86_64-pc-windows-msvc/release/bundle/nsis/Dandy Harness_0.1.0_x64-setup.exe`.
L'installation est par utilisateur, dans `%LOCALAPPDATA%/Dandy Harness`.
Le script prépare les paquets locaux vérifiés et leurs dépendances de production
dans un dossier isolé, puis embarque ce moteur avec Node. Le checkout est requis
pour construire le paquet, pas pour l'exécuter. `-EngineDirectory` accepte un
dossier de production déjà préparé pour éviter de refaire cette étape.

En mode développement, le lanceur Rust utilise `apps/cli/lib/bin.js`. Après une modification des sources de la CLI, reconstruire cet artefact pour que le prochain lancement utilise la modification. La distribution embarquée utilise `engine/node_modules/@deepseek-ai/dsh/lib/bin.js`.

Dans un terminal développeur Visual Studio x64 avec Rust MSVC :

```powershell
cargo +stable-x86_64-pc-windows-msvc build --locked --target x86_64-pc-windows-msvc
```

Les intégrations Electron spécifiques ne sont pas portées. L’icône provient du fork Harness, dont la licence accompagne le fichier dans `icons/`.

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
