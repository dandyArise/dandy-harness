# App-boot : profil détaillé et essai de cache

## Attribution

Profil CPU Node existant `target/startup-current.cpuprofile`, mêmes artefacts de
départ que la précédente passe. Temps inclusifs, non additionnables :

| Chemin | ms échantillonnées |
|---|---:|
| app-boot total | 1 189 |
| Include._read → _scheduleApply (application des plugins) | 761 |
| Interception ESM wrapped → adapted | 412 / 394 |
| createRuntimeResolution → collectInstallationScopePackages | 148 / 147 |
| Prévalidation des entrées | 61 |

`_read` inclut le travail lancé par l'application des plugins : ce n'est pas
761 ms de lecture du seul YAML. Dans `adapted`, les feuilles dominantes sont
internalModuleStat (80,5 ms), readPackageJSON (63,8), getPackageScopeConfig (53,1),
lstat (45,9). La majeure partie appartient à la résolution native Node.

## Essai réalisé et retiré

Cache des sondes d'existence de package.json, limité à une seule traversée du
graphe d'installation. Aucun cache inter-démarrages ni inter-rechargements.
Comparaison des bundles app-boot avant/après, le reste des artefacts identique,
exécutable Release. Six processus neufs, ordre A/B/B/A/A/B. Pas de compilation
concurrente. Les snapshots des bundles sont restaurés entre essais.

| Variante | Chat éditable ms | Moteur prêt ms | Mémoire privée totale Mio |
|---|---:|---:|---:|
| Avant 1 | 4 756,14 | 4 154,78 | 768,83 |
| Après 1 | 2 368,97 | 1 870,26 | 774,65 |
| Après 2 | 2 339,68 | 1 897,88 | 766,93 |
| Avant 2 | 2 312,28 | 1 862,03 | 769,79 |
| Avant 3 | 2 329,97 | 1 864,85 | 768,40 |
| Après 3 | 2 274,95 | 1 858,87 | 768,19 |

Médianes chat : **2 329,97 → 2 339,68 ms**. Aucun gain démontré.
Le premier lancement est atypiquement lent ; même son exclusion ne change pas
la décision (avant chaud : 2 321,13 ms). Mémoire privée médiane :
768,83 → 768,19 Mio, différence négligeable. Mesure de mémoire : cinq derniers
échantillons, Tauri/Node/WebView2 et descendants, pas RAM physique exclusive.
Trois essais par variante ne permettent pas de revendiquer un p95 fiable.
Données : `target/appboot-[1-6]-*.json` ; script `benchmarks/startup.ps1`.

Le cache a été retiré, sans retirer les optimisations qui existaient avant cette
passe dans profile.ts/resolver.ts. Les artefacts du comportement initial sont
reconstruits. Pas de nouveau portage Rust ni de gain revendiqué.

## Contrôles

- Profil/résolution/service : 282 tests réussis ; un scénario bloqué avant son
  exécution par EPERM à la création d'un lien symbolique de fichier Windows.
- Test ajouté : un nouvel inventaire voit une dépendance ajoutée puis retirée,
  réussi. Conservé pour protéger les prochaines optimisations de ce chemin.
- build:artifacts après retrait : 8 contrôles réussis.
- build:check après reconstruction finale : 2 contrôles réussis.

## Parcours réel Tauri

Vérifié avec le modèle sélectionné Inkling 2 Mini, sans changer le modèle chargé :
- Discussion libre : « Combien font 17 + 25 ? » donne 42, terminé en 1 seconde.
- Mode standard, nouvelle session de lecture seule : lecture demandée de
  target/boot-tool-check/preuve.txt, dont le nonce n'était pas fourni dans le prompt.
  Le modèle restitue exactement « Validation app-boot : 7361a39f-9080-4dc9-93b8-53089eb4c5b2 ».
  Tauri affiche terminé en 1 min 41 s, 28.7K jetons et 2 étapes.
  Cela valide le parcours de lecture, mais la latence reste élevée pour cette tâche.
- Défaut distinct observé sur une réponse antérieure : balises think visibles
  dans Discussion libre. Ce défaut n'est pas corrigé par cette passe app-boot.

## Suivi : Inkling et balises think (21 novembre, 23 h 38)

Les traces persistées de la session de lecture isolent la latence :

| Mesure | Premier essai | Retest à chaud | Retest avec normalisation complète |
|---|---:|---:|---:|
| Étape 1, avant premier fragment | 94 943 ms | 2 402 ms | 3 084 ms |
| Étape 1, total | 96 995 ms | 3 859 ms | 4 625 ms |
| Outil read | 23 ms | 14 ms | 23 ms |
| Étape 2, total | 4 067 ms | 5 534 ms | 5 364 ms |
| Entrée étape 1 | 14 102 tokens | 14 621 tokens | 15 124 tokens |

Le retard initial précède les tokens de réponse, pas l'exécution du fichier.
Un préremplissage froid du contexte est l'explication probable ; les journaux
serveur disponibles ne couvrent pas cet horaire et ne permettent pas d'exclure
une attente côté moteur. Ce n'est pas une preuve d'accélération due au patch.
Le modèle Inkling 2 Mini et ses réglages LM Studio n'ont pas changé.

Le retest final dans Tauri restitue le nouveau nonce en 10 secondes. Les traces
contiennent un vrai appel read et son résultat. La réponse ne contient plus de
balises think ; les blocs de raisonnement sont séparés du texte final.
La normalisation est appliquée dans l'adaptateur, pas dans le rendu Markdown.
Les anciennes réponses déjà enregistrées ne sont pas réécrites.
Discussion libre retestée ensuite dans Greeting avec le même Inkling 2 Mini :
réponse en français en 5 secondes, sans balises internes visibles.

Vérification : 74 tests ciblés réussis puis le fichier de normalisation enrichi
repasse ses 5 tests (75 cas distincts au total). build:artifacts : 8 contrôles
réussis ; build:check : 2 contrôles réussis. Aucun commit/push dans cette passe.
