# Ground Textures

Структура текстур для поверхности земли:

- `dirt/` — базовые грунтовые/сухие наборы.
- `mud/` — грязевые/влажные наборы.
- `snow/` — снежные/зимние наборы.

Рекомендуемый формат набора:

- `<name>_diff_4k.(jpg|png)` — albedo/base color.
- `<name>_nor_gl_4k.(jpg|png)` — normal map (OpenGL).
- `<name>_arm_4k.(jpg|png)` — AO/Roughness/Metallic packed map.
- `<name>_ao_4k.(jpg|png)` — отдельная AO (опционально).

Пример:

- `dirt/coast_sand_rocks_02/`
