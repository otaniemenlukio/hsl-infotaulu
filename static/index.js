
let pingCount = 0;

const connect = () => {
    const uri = location.host.includes('localhost') ? 'ws://' + location.host + '/connect' : 'wss://' + location.host + '/connect';
    const ws = new WebSocket(uri);
    console.log(`> Connected to ${uri}`);

    setInterval(() => {
        if (pingCount >= 5) {
            window.location.reload();
        }

        if (pingCount > 0) {
            console.warn(`Missed ping (#${pingCount})`);
        }

        pingCount += 1;
    }, 2000);

    return ws
} 


const initialize = () => {
    const ws = connect();
    const container = document.getElementById('CONTAINER');
    ws.onmessage = (({ data }) => {
        container.innerHTML = data;
        pingCount = 0;
    });
}



document.addEventListener('DOMContentLoaded', initialize);