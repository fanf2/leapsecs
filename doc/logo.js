const TAU = Math.PI * 2;

// clockwise from the top
function angle(theta) {
    return TAU * 3 / 4 + theta;
}
function sin(theta) {
    return Math.sin(angle(theta));
}
function cos(theta) {
    return Math.cos(angle(theta));
}

function circle(ctx, r) {
    ctx.beginPath();
    ctx.arc(0,0, r, 0,TAU, false);
    ctx.stroke();
}

function hand(ctx, base, length, tip) {
    ctx.save();
    ctx.lineWidth = 1;
    ctx.strokeStyle = "white";
    ctx.fillStyle = "black";
    ctx.beginPath();
    ctx.arc(0,0, base, 0, TAU/2, false);
    ctx.lineTo(-tip, -length);
    ctx.lineTo(0, - tip - length);
    ctx.lineTo(+tip, -length);
    ctx.lineTo(+base, 0);
    ctx.fill();
    ctx.stroke();
    ctx.restore();
}

function draw() {
    const canvas = document.querySelector('canvas');
    const ctx = canvas.getContext('2d');
    ctx.translate(canvas.width / 2, canvas.height / 2);

    const inner = 350;
    const outer = 370;

    const bignums = 300;
    const biggernums = 290;
    const smolnums = 390;

    const innerer = inner - 6;
    const outerer = outer + 6;

    const sechand = outerer;
    const minhand = innerer;
    const hourhand = 250;

    const ticks = 61;
    const tick = TAU / ticks;

    ctx.save();
    ctx.lineWidth = 2;
    circle(ctx, inner);
    circle(ctx, outer);
    ctx.restore();

    ctx.save();
    ctx.lineWidth = 3;
    ctx.beginPath();
    for (let i = 0; i < ticks; i++) {
	const x = cos(i * tick);
	const y = sin(i * tick);
	ctx.moveTo(inner * x, inner * y);
	ctx.lineTo(outer * x, outer * y);
    }
    ctx.stroke();
    ctx.restore();

    ctx.save();
    ctx.lineWidth = 5;
    ctx.beginPath();
    for (let i = 0; i <= 55; i += 5) {
	const x = cos(i * tick);
	const y = sin(i * tick);
	ctx.moveTo(innerer * x, innerer * y);
	ctx.lineTo(outerer * x, outerer * y);
    }
    ctx.stroke();
    ctx.restore();

    ctx.save();
    ctx.font = "80px Monaco";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.beginPath();
    for (let i = 0; i < 12; i++) {
	const r = 1 <= i && i <= 9 ? bignums : biggernums;
	const x = r * cos(i * 5 * tick);
	const y = r * sin(i * 5 * tick);
	ctx.fillText(i == 0 ? "12" : i == 10 ? "1O" : i, x, y);
    }
    ctx.stroke();
    ctx.restore();

    ctx.save();
    ctx.font = "20px Monaco";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    for (let i = 0; i <= 55; i += 5) {
	ctx.save();
	ctx.rotate(i * tick);
	let n = i == 0 ? "61" : i + "";
	n = n.replace("0", "O");
	ctx.fillText(n, 0, - smolnums);
	ctx.restore();
    }
    ctx.stroke();
    ctx.restore();

    ctx.save();
    ctx.rotate(- TAU * 1/366);
    hand(ctx, 20, 250, 8);
    ctx.restore();

    ctx.save();
    ctx.rotate(59 * tick);
    hand(ctx, 15, 340, 5);
    ctx.restore();

    const r = 11;
    const tip = 1;
    const base = 3;
    const head = 390;
    const tail = 125;

    ctx.save();
    ctx.rotate(60 * tick);
    ctx.fillStyle = "#999";
    ctx.beginPath();
    ctx.arc(0,0, r, 0, TAU, false);
    ctx.moveTo(+base, 0);
    ctx.lineTo(+base, tail);
    ctx.lineTo(-base, tail);
    ctx.lineTo(-base, 0);
    ctx.lineTo(-tip, -head);
    ctx.lineTo(+tip, -head);
    ctx.closePath();
    ctx.fill();
    ctx.restore();
}
