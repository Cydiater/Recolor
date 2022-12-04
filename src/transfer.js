import init, { transfer } from 'recolor';

onmessage = function(e) {
    init().then(() => {
        const result = transfer(e.data[0], e.data[1], e.data[2]);
        postMessage(result);
    })
}
