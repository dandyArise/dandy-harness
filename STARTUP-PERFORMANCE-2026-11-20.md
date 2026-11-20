# Démarrage et mémoire — 20 novembre 2026

## Changement conservé

Le lanceur démarre Node avant de construire WebView2, afin de chevaucher les deux
initialisations. Il conserve immédiatement le handle supervisé. Les échecs Node
restent affichés après création de la fenêtre, et les mécanismes d’arrêt existants
ne changent pas. Aucun plugin ni protection n’a été désactivé.

## Comparaison mesurée

Windows, Node 24.22.3, exécutable Rust debug, même profil utilisateur et même session,
Glimmer v7 conservé. Une chauffe de la variante puis huit lancements dans l’ordre
avant/après/après/avant/avant/après/après/avant. Aucune compilation ni génération LLM
pendant cette série ; autres applications Windows présentes, caches disque non purgés.

| Médiane (4 essais par variante) | Avant | Après |
|---|---:|---:|
| Entrée Rust → éditeur visible et éditable, après deux frames | 2 495,07 ms | 2 298,28 ms |
| Entrée Rust → moteur prêt | 2 056,36 ms | 1 862,29 ms |
| Création fenêtre terminée, depuis l’entrée Rust | 234,71 ms | 267,43 ms |
| Mémoire privée totale à disponibilité + environ 5 s | 809,92 Mio | 807,87 Mio |
| Somme des working sets au même point | 1 079,49 Mio | 1 078,63 Mio |
| Pic échantillonné de mémoire privée pendant la fenêtre mesurée | 809,99 Mio | 807,87 Mio |

Gain médian jusqu’au chat : **196,79 ms, soit 7,9 %**. Plages observées :
avant 2 487–2 730 ms ; après 2 268–2 307 ms. Seulement quatre mesures par variante :
pas de revendication de p95 robuste. Le coût de lancement avant création du processus
est d’environ 5 ms ; le temps natif ne comprend pas le chargement initial de l’EXE
avant `main`. Ce n’est pas un démarrage à froid après redémarrage de Windows.

### Mémoire par famille, après optimisation

- Tauri : 4,62 Mio privés.
- Node : 310,91 Mio privés.
- WebView2, tous ses processus : 423,17 Mio privés.
- Autres descendants (notamment le service Python) : 69,20 Mio privés.

Les médianes par famille ne s’additionnent pas exactement à la médiane du total.
La somme des working sets peut compter plusieurs fois les pages partagées ; elle
ne représente pas une RAM physique exclusive. Les octets privés mesurent l’engagement
privé, pas uniquement la RAM résidente. **Aucun gain mémoire significatif démontré**
avec cette modification (~2 Mio de différence seulement).

## Profil et essai rejeté

Un profil CPU séparé pointe le chargement Cordis/app-boot : environ 1 362 ms inclusifs
dans Cordis, 1 188 ms dans app-boot et 889 ms dans le loader. Ces temps sont imbriqués,
ne doivent pas être additionnés et ne constituent pas des mesures individuelles de
chaque plugin. Résolution, ouvertures de fichiers et initialisation dominent davantage
que la compilation seule.

Le cache de compilation Node a été essayé sur trois lancements, avec écriture
explicite du cache. Environ 7,5 Mo et 1 251 fichiers créés ; aucun gain convaincant
sur le chat (~2,75 s après chauffe) ni sur la mémoire. L’activation et le timer de
flush ont été retirés. Le cache expérimental reste local sous `data/node-compile-cache`
et n’est pas utilisé automatiquement. API consultée :
[documentation Node](https://nodejs.org/api/module.html#module-compile-cache).

## Reproduire

Le script ne lance aucune génération et ne saisit aucun message. Il détecte le
champ `[data-composer-input][contenteditable=true]`, visible et non désactivé,
attend deux frames puis transmet un marqueur par le titre, uniquement en diagnostic.
Les jalons Rust fournissent le temps ; la détection PowerShell sert au prélèvement
mémoire et peut arriver plus tard. Le script suit les descendants et termine uniquement
son propre EXE à la fin ; le Job Object nettoie le moteur. Fermer Harness au préalable.

```powershell
cargo +stable-x86_64-pc-windows-msvc build --locked --target x86_64-pc-windows-msvc
$exe = "$PWD\target\x86_64-pc-windows-msvc\debug\dandy-harness.exe"
# Chauffe non incluse dans la comparaison
.\benchmarks\startup.ps1 -Exe $exe -Output "$PWD\target\startup-warm.json" -Runs 1
$order = @('before','after','after','before','before','after','after','before')
for ($i = 0; $i -lt $order.Count; $i++) {
  $variant = $order[$i]
  .\benchmarks\startup.ps1 -Exe $exe -Output "$PWD\target\startup-$i-$variant.json" -Label $variant -Runs 1 -Serial:($variant -eq 'before')
}
```

Résultats numériques de la série : `benchmarks/results/startup-2026-11-20.json`.
Échantillons complets et profil CPU locaux sous `target/startup-*`.

## Vérification réelle

- Build, format et Clippy `-D warnings` réussis.
- Tests Rust : 8 réussis, 1 benchmark ignoré.
- Dans Tauri sans diagnostic : saisie visible, envoi actif, modèle Glimmer v7 inchangé.
- Premier outil : lecture d’un fichier temporaire contenant une valeur aléatoire
  absente du prompt ; appel « Lire des fichiers » observé, valeur restituée exactement,
  tour terminé en 5 secondes. Aucun fichier utilisateur modifié par le modèle.
- Aucun survivant signalé après les arrêts des huit processus de benchmark.

Cette passe optimise le chemin critique de lancement ; elle ne réduit pas le travail
de chaque plugin, ne réécrit pas le moteur et n’accélère pas la génération du LLM.
