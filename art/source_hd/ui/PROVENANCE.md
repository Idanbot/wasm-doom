# BLACKSITE interface art

Generated 2026-09-23 with the built-in `image_gen` tool. All source images were visually inspected. No external source images were used. The share card uses the generated menu environment as a reference.

## Outputs

- `menu-reactor.png`: untouched generated master, 1672 × 941; runtime `/game/ui/menu-reactor.webp`, quality 86.
- `hud-panel.png`: untouched generated master, 2172 × 724; runtime `/game/ui/hud-panel.webp`, quality 92. Actual alpha transparency outside panel; opaque dark interior for live HTML numbers and labels.
- `share-card.png`: untouched generated master; runtime `/og.jpg`, center-cropped/resized to 1200 × 630, JPEG quality 90.

Runtime conversions use ImageMagick only for standard encoding and share-card dimensions. Menu titles, controls, HUD values and settings remain live interface elements rather than baked image content.

## Prompts

### Menu environment

Use case: stylized-concept
Asset type: production main menu background for BLACKSITE, a gritty industrial science fiction FPS
Primary request: cinematic wide 16:9 bunker reactor corridor background, no text or interface. A heavy blast door and glimpses of a reactor containment chamber on the RIGHT half, layered pipes and ribbed steel passage, wet scratched metal floor, wisps of ventilation steam. LEFT 45 percent is very dark low-detail graphite wall and shadow, usable negative space for live menu text. Strong architectural depth and believable worn industrial surfaces.
Style: high quality grounded cinematic game environment render, oppressive military research facility, tactile metal, atmospheric not blurry.
Palette: graphite #080d11, ash metal #dbe6e6, sparse amber #f39b42 emergency strip lighting, small cold cyan #78ced0 electronics.
Constraints: landscape wide frame, no people, no guns, no words, no letters, no logos, no watermark, no UI elements. Keep bright lights and focal detail concentrated to the right. Render one complete background.

### HUD panel

Use case: stylized-concept
Asset type: production raster HUD panel skin for a gritty industrial FPS, to use behind live HTML health and ammo numbers
Primary request: ONE horizontal rectangular industrial metal panel, front facing orthographic perfectly flat no perspective, aspect approximately 3 to 1. Genuinely TRANSPARENT background outside the panel, preserved alpha, NOT a checkerboard. A thin bevel-edged scratched charcoal gunmetal frame, clipped corners, subtle bolts, fine tactile surface, tiny restrained amber status-light accents at top left and cyan at lower right. Main central interior takes 85% of panel area and is empty flat dark graphite #080d11 with very subtle matte texture. The frame is narrow and sleek enough to not dominate a videogame HUD.
Style: high quality realistic worn military science fiction equipment display surround, finely detailed edge only; extremely legible empty center.
Palette: graphite #080d11, desaturated steel #819298, sparse amber #f39b42 and cyan #78ced0 accents.
Constraints: one single panel centered filling nearly the whole wide image, transparent outside only, no screen contents, no words, no numbers, no symbols, no logos, no buttons, no visualized complete UI, no extra panels, no watermark, no heavy grunge in empty center. Provide a production ready transparent PNG.

### Share card

Use case: ads-marketing
Asset type: game social share card for BLACKSITE, wide landscape near 1.9:1 aspect ratio
Input image: style and environment reference (menu-reactor.png).
Primary request: a cinematic share card matching this exact industrial bunker-reactor environment, amber warning lights, cyan reactor, graphite military metals. Preserve dramatic right-half reactor/door composition and dark left space. In the LEFT half, render a bold very large original angular condensed science fiction stencil wordmark reading exactly "BLACKSITE" in ash white with slightly worn edges and tight tracking. Below in much smaller clean uppercase condensed lettering: "CONTAINMENT HAS FAILED". Add one fine amber horizontal rule under subtitle. Typography is the hero and must be readable at thumbnail size. No other words, no logos, no UI, no people, no watermark. Keep mood dark, grounded and tense.
