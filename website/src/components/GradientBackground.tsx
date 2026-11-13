/**
 * Fixed full-viewport backdrop: the solid theme background, three slowly
 * drifting monochrome gradient blobs and a grain layer on top.
 * All colors come from the theme file; nothing is tinted here.
 */
export function GradientBackground() {
    return (
        <div className="bg" aria-hidden="true">
            <div className="bg__blob bg__blob--a" />
            <div className="bg__blob bg__blob--b" />
            <div className="bg__blob bg__blob--c" />
            <div className="bg__noise" />
        </div>
    );
}
