# BLACKSITE enemy art direction

## Visual thesis

BLACKSITE enemies are readable industrial silhouettes first and detailed character renders second. They share a cold, worn facility material language, while shape and one controlled accent color communicate faction and combat role at a glance.

The runtime target is a 128 px frame. Large forms must therefore carry the design: helmet profile, shoulder width, weapon mass, stance, limb count, and one luminous focal point. Fine bolts, seams, fabric weave, scratches, hoses, and optics reward close viewing but never replace those forms.

## Shared rendering rules

- Use a straight-on, orthographic-like camera and a stable bottom-center gameplay anchor.
- Use cool overhead industrial light, hard value separation, and one restrained warm rim.
- Keep black materials separated with graphite, steel, olive, bone, or construction yellow panels rather than crushing everything into the same value.
- Reserve emissive color for sensors, reactors, chemical tanks, and threat telegraphs.
- Keep the whole subject and all equipment inside the canvas. Preserve real transparency in gaps between limbs, weapons, rotors, and cables.
- Show wear through chipped edges, soot, grease, rubbed paint, stained fabric, and scratched ceramic. Avoid clean showroom surfaces.
- Avoid text, logos, scenery, floor shadows, detached particles, and baked-in muzzle effects.

## Silhouette families

- **Directorate infantry:** Rifleman, Breacher, Gunner, Hazmat, and Marksman share charcoal composites, functional field gear, and amber optics. Their stance and weapon mass identify role.
- **Industrial machines:** Loader and Hound use construction geometry, exposed actuators, grease, chipped enamel, and warning yellow. They must not resemble armored humans.
- **Airborne machines:** Hornet and Martyr use wide compact silhouettes. Hornet has amber surveillance optics; Martyr concentrates red light around its unstable reactor.
- **Containment failures:** Subject, Spitter, and Vatbrute combine pallid tissue, dirty ceramic restraints, surgical hardware, hoses, and limited red or acid-green emissions.
- **Command:** VEYRAN uses the same industrial materials at ceremonial scale: crown profile, articulated mantle, armored coat panels, and one concentrated amber reactor.

## Next art improvements

1. Generate action-specific 2x2 masters for movement, firing, pain, reload/charge, death, and special actions. The current anchored deformation keeps the game responsive, but hand-authored poses would improve limb motion and weapon handling.
2. Add eight-view rotations for close-range enemies and bosses. Front-only billboards are readable, but side and rear views would make strafing and flanking feel more physical.
3. Separate body animation from effects. Muzzle flashes, acid spray, electricity, reactor pulses, and death debris should use dedicated overlays so effects can scale and light the scene independently.
4. Add damage variants at health thresholds for tanks and bosses: cracked armor, severed hoses, flickering optics, exposed reactors, and leaking coolant.
5. Give each level family a restrained environmental tint in the renderer instead of baking map color into sprites. This keeps enemy identity stable while grounding them in each location.
