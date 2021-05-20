var canvas;
var context;
var cw, ch;
var zoom = 1;
var panx = 0;
var panz = 0;

const SEG_SIZE = 8 * 16;
const CHUNK_HEIGHT = 255;

var segments = new Map();

var [mlx, mly] = [0, 0];
var mouse_down = false;

function isometric_map_coord(x, y, z) {
    let [x2, y2, z2] = [x * 16, y * 16, z * 16];
    return [(x2 - z2) / 2, y2 / 2 + (x2 + z2) / 4];
}

window.addEventListener("load", () => {
    canvas = document.createElement("canvas");
    context = canvas.getContext("2d");
    window.document.body.appendChild(canvas);

    window.onresize = resize;
    window.document.onmousedown = (ev) => {
        mouse_down = true;
    };
    window.document.onmouseup = (ev) => {
        mouse_down = false;
    };
    window.document.onmousemove = (ev) => {
        var [mx, my] = [ev.clientX, ev.clientY];
        if (mouse_down) {
            let dx = (mx - mlx) * -0.5;
            let dy = (my - mly) * 1;
            panx += (dy - dx);
            panz += (dy + dx);
        }

        mlx = mx;
        mly = my;
    };

    resize();
    draw_loop();
});

function resize() {
    cw = window.screen.availWidth;
    ch = window.screen.availHeight;
    canvas.style.position = "absolute";
    canvas.style.top = "0px";
    canvas.style.left = "0px";
    canvas.style.width = `${cw}px`;
    canvas.style.height = `${ch}px`;
    canvas.width = cw;
    canvas.height = ch;
}

function segment_name(x, z) {
    return `seg.${x}.${z}.seg.png`;
}

function load_segment(x, z) {
    var image = document.createElement("img");
    image.src = `./prerendered/${segment_name(x, z)}`;
    segments.set(segment_name(x, z), image);
}

function get_segment(x, z) {
    let segname = segment_name(x, z);
    if (!segments.has(segname)) load_segment(x, z);
    return segments.get(segname);
}

function draw_loop() {
    context.fillStyle = "black";
    context.fillRect(0, 0, cw, ch);
    context.imageSmoothingEnabled = true;

    context.save();
    context.transform(0.2, 0, 0, 0.2, 0, 0);

    for (let x = 0; x < 4; x++) {
        for (let z = 0; z < 4; z++) {
            let seg = get_segment(x - 4, z);
            let [sx, sy] = isometric_map_coord(
                x * SEG_SIZE + panx,
                0,
                z * SEG_SIZE + panz
            );

            context.drawImage(
                seg,
                sx,
                sy,
                SEG_SIZE * 16,
                (SEG_SIZE + CHUNK_HEIGHT / 2) * 16
            );
        }
    }

    context.restore();
    requestAnimationFrame(draw_loop);
}
