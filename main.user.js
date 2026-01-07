// ==UserScript==
// @name        Send to Makereal Cafe
// @namespace   makereallabs.org
// @match       https://www.youtube.com/*
// @grant       none
// @version     2.0
// @author      -
// @description Add a button to YouTube for sending the current music to Makereal Cafe!
//              (https://pi.makereallabs.org) Move your mouse to the middle-right
//              of the window to show it.
// @run-at      document-start
// ==/UserScript==

let icon = `<svg width="58.96mm" height="58.96mm" version="1.1" viewBox="0 0 58.96 58.96" xml:space="preserve" xmlns="http://www.w3.org/2000/svg"><g transform="matrix(.03342 0 0 .03349 -4.32 -4.531)"><rect x="129.3" y="135.3" width="1764" height="1760" fill="#1e1e78" stroke-opacity=".5" stroke-width="1.134"/><path d="m522.6 1295a203.2 203.2 0 0173.9-122.8" fill="none" stroke="#fff" stroke-linecap="round" stroke-width="9.009"/><path transform="scale(.75)" d="m1627 2523c-52.24-13.2-114.4-52.47-145.6-91.95-13.68-17.3-20.06-35.51-20.81-51.16-.054-1.134-1.845-231.9-1.841-233 .082-21.68 11.27-38.06 28.81-38.16 462-2.614 0 0 432-2.722 19.84-.125 39.07.3784 43.83-.034 15.43-1.335 40.63 7.037 61.16 21.91 23.16 16.78 32.95 9.779 33.31 35.45.3219 22.7-3.643 32.29-21.12 51.11l-8.768 9.439 13.49 20.32c10.77 16.22 13.79 24.51 14.99 41.11 2.011 27.8-3.608 42.89-24.46 65.7-25.31 27.69-55.83 40.22-114.9 47.19-15.58 1.836-17.59 3.283-27.98 20.17-26.51 43.08-70.1 78.45-119.1 96.66-25.79 9.582-34.42 10.96-75.58 12.05-32.82.8712-52.61-.3193-67.38-4.054z" fill="#784421"/><path transform="scale(.75)" d="m1040 2407c-33.5-3.704-80.46-17.6-110.8-32.79-76.06-38.11-128.8-103.7-148.9-185.1-4.116-16.71-7.868-30.82-8.337-31.35-.4694-.5349-11.14 1.452-23.71 4.415-12.57 2.963-37 5.411-54.28 5.44-168.6.2834-332.6-153.1-366.1-342.4l-5.918-33.4-20.76-14.56c-23.05-16.17-38.25-36.62-49.61-66.75-10.56-28.01-8.387-71.99 5.032-101.7l9.439-20.88-10.2-14.86c-26.59-38.74-51.76-102.3-60.97-154-2.079-11.66-3.765-43.83-3.747-71.48.0376-58.36 9.21-106.9 29.7-157.2 25.48-62.49 76.58-125.2 125.2-153.6 14.31-8.358 22.76-15.44 21.34-17.87-1.295-2.217-9.169-12.88-17.5-23.69-33.1-42.97-54.75-94-63.96-150.8-6.845-42.21-6.79-57.98.3466-99.65 11.33-66.14 39.38-124 83.35-172 29.44-32.11 58.9-53.64 102-74.5 44.11-21.37 85.46-32.19 132.5-34.67l35.91-1.894 12.34-19.66c23.83-37.97 64.63-60.46 116.4-64.11l29.53-2.084 22.58-28.74c12.42-15.8 35.45-38.94 51.18-51.4 162.4-128.7 402-80 511.5 104l16.83 28.3 2.694 84.35c1.481 46.39 3.727 158.3 4.99 248.8 2.153 154.1 1.893 165.2-4.149 176.7-18.94 36-63.5 36.65-84.25 1.229-5.724-9.773-6.645-30.07-9.795-215.9-5.02-296.1-2.866-265.4-20.41-291.2-33.06-48.63-90.56-88.98-147.3-103.4-27.8-7.057-91.55-8.025-118.7-1.802-34.5 7.906-76.12 30.5-103.1 56-13.68 12.91-28.14 28.75-32.13 35.21l-7.254 11.74 12.3 25.67c21.82 45.53 20.08 90.18-5.269 135.1l-10.64 18.85 47.89 70.76c52.15 77.06 59.99 95.12 50.64 116.6-2.595 5.969-30.59 42.22-62.22 80.55-31.62 38.33-57.53 70.9-57.56 72.38-.0332 1.474 60.36 2.681 134.2 2.681 91.85 0 136.7 1.261 142.1 3.992 22.1 11.3 27.12 38.01 10.49 55.8l-9.829 10.51-81.89 2.378c-45.04 1.308-122.2 2.385-171.6 2.394l-89.67.017-141.8 172.4c-77.98 94.84-146.8 177.2-152.8 183-25.3 24.13-66.1 3.037-59.62-30.82 1.559-8.157 19.37-32.71 56.74-78.22 29.96-36.48 127.7-155.2 217.1-263.9 89.44-108.7 168.9-205.5 176.5-215.2l13.91-17.58-35.36-52.09c-19.45-28.65-39.39-56.92-44.31-62.83-8.93-10.72-9.013-10.74-30.38-6.921-25.01 4.464-59.03-1.192-84.1-13.98-22.14-11.29-47.72-36.73-59.38-59.05l-10.19-19.49-28.74-.008c-101.1-.0299-191.5 70.81-219.6 172.1-8.835 31.88-9.678 84.83-1.842 115.8 15.06 59.43 53.86 111.3 110.3 147.5 15.49 9.92 31.16 22.58 34.82 28.14 13.71 20.79 4.728 55.49-17.26 66.66-5.575 2.832-25.21 5.838-44.45 6.804-47.09 2.364-73.06 14.41-108.3 50.21-84.89 86.3-98.05 258.9-27.72 363.6l13.2 19.66 21.81-1.192c12.65-.6912 31.93 1.346 45.9 4.85 13.25 3.323 25.34 4.664 26.87 2.979 1.529-1.684 63.62-81.87 138-178.2 200.4-259.6 203.8-263.9 216.7-271.6 11.24-6.781 18.19-7.148 135.2-7.148h123.4l24.21 11.92c27.02 13.3 49.58 37.75 55.93 60.63 2.206 7.944 3.684 65.09 3.696 142.9.022 144.8-.3616 147.5-23.32 161.7l-13.44 8.308-391.7 3.309-10.93-10.93c-15.28-15.28-15.47-36.54-.4409-50.58 10.35-9.672 11.28-9.828 72.38-12.15 34.04-1.291 111.2-2.352 171.5-2.358 60.29-.01 112.2-.9872 115.3-2.18 4.591-1.762 5.654-8.376 5.654-35.19v-33.02l-50.93-2.33c-28.01-1.282-65.17-2.336-82.56-2.343-31.2-.013-31.8-.1787-43.78-12.17-13.36-13.36-15.36-24.89-7.096-40.87 8.871-17.16 15.7-18.43 104-19.37l82.21-.8776v-36.95c0-36.05-.2431-37.15-10.03-45.57l-10.03-8.626h-212.7l-9.076 9.829c-13.4 14.51-324.7 418.5-326.9 424.2-1.034 2.695.6571 9.806 3.758 15.8 3.101 5.997 8.172 21.82 11.27 35.17 6.287 27.1 3.456 51.83-9.353 81.71-8.976 20.94-36.39 49.88-59.76 63.09-15.27 8.63-18.76 12.38-18.76 20.1 0 10.6 10.11 47.43 19.77 72.02 19.66 50.05 69.06 112.6 114.5 145.1 78.18 55.79 167.7 62.25 227.8 16.46 10.19-7.757 22.04-15.91 26.34-18.13 19.34-9.965 49.62.7217 60.24 21.26 4.368 8.446 4.9 20.14 2.963 65.08-2.178 50.52-1.75 56.62 5.494 78.39 14.63 43.95 45.1 77.28 91.93 100.6 35.14 17.46 64.61 23.78 111 23.78 82.64 0 160.3-31.38 218-88.07 37.35-36.68 34.19-7.671 29.48-270.6-3.975-222.2-3.882-231.2 2.509-244.1 18.2-36.62 63.53-37.97 84.31-2.51 5.472 9.342 6.598 29.73 9.754 176.5 4.727 219.9 4.953 324.9.7227 336-4.776 12.56-42.12 58.76-63.67 78.76-55.26 51.3-129.6 89.61-203.2 104.6-34.06 6.964-86.99 9.391-121.5 5.574z" fill="#fff"/><path transform="scale(.75)" d="m1350 1607c-6.293-3.837-14.25-12.28-17.68-18.75-6.598-12.45-6.518-10.17-14.06-398.1l-2.298-118.2-160.9-192.7c-190.2-227.9-192.2-230.4-192.2-247 0-24.94 27.17-40.3 51.59-29.17 9.047 4.122 385.3 452.5 394.9 470.6 5.31 10.01 8.822 128.9 11.58 392l1.199 114.4-8.324 11.67c-15.82 22.18-42.21 28.49-63.88 15.28z" fill="#fff"/><path transform="scale(.75)" d="m1866 2073c-17.76.9215 5.267-80.6 18.35-121.2 27.41-84.98 84.81-176.6 145.2-231.7 8.225-7.507 30.23-25.04 48.91-38.96 186.3-138.9 304.4-323.1 339.5-529.5 53.37-314.1-101.1-619.6-390.4-772.4-150.1-79.29-312-111.5-495.2-98.62-58.42 4.112-62.34 3.984-73.26-2.391-36.16-21.12-35.66-65.91.9515-84.94 27.24-14.16 185-17.19 277.8-5.336 315.2 40.27 578 221.5 707 487.7 39.88 82.26 63.46 160.8 74.93 249.6 6.829 52.85 6.975 156.2.2915 206.2-21.23 158.7-90.2 317.6-193.9 446.7-51.03 63.51-113.2 122.7-194.4 185.1-59.17 45.44-95.3 88.11-126.8 149.8-58.98 216.5-1.504 152.8-139 160z" fill="#efcd07"/><path transform="scale(.75)" d="m1627 2523c-52.24-13.2-114.4-52.47-145.6-91.95-13.68-17.3-20.06-35.51-20.81-51.16-.054-1.134-1.845-256.6-1.841-257.7.082-21.68 11.27-37.96 28.81-38.16 19.09-.2139 32.16 6.969 41.48 38.13.3182 1.064 3.175 256 3.875 256.9 24.6 29.39 44.94 45.91 71.5 58.5 59.35 28.12 124.6 25.84 174.7-6.122 23.9-15.22 41.05-35.65 55.51-66.11 17.93-37.77 22.52-40.56 66.68-40.7 42.59-.1293 71.47-9.286 80.58-25.56 4.901-8.746 4.791-10.23-1.252-16.9-11.54-12.75-42.21-21.76-74.04-21.76-41.61 0-56.96-10.71-57.37-40.02-.1507-10.92 2.064-16.63 9.233-23.81 8.787-8.798 12.17-9.572 49.57-11.33 96.68-4.54 100.5-36.98 4.398-37.37-45.93-.1879-55.07-2.947-62.29-18.8-5.242-11.5-4.804-20.83 1.557-33.13 7.645-14.78 24.03-18.31 67.89-14.59 19.77 1.675 39.36 1.66 43.83-.034 10.2-3.862 40.63 7.037 61.16 21.91 23.16 16.78 32.95 34.44 33.31 60.11.3219 22.7-3.643 32.29-21.12 51.11l-8.768 9.439 13.49 20.32c10.77 16.22 13.79 24.51 14.99 41.11 2.011 27.8-3.608 42.89-24.46 65.7-25.31 27.69-55.83 40.22-114.9 47.19-15.58 1.836-17.59 3.283-27.98 20.17-26.51 43.08-70.1 78.45-119.1 96.66-25.79 9.582-34.42 10.96-75.58 12.05-32.82.8712-52.61-.3193-67.38-4.054z" fill="#cfcfcf"/><circle cx="722.4" cy="1332" r="179.5" fill="#fff" stroke-width="2.603"/><circle cx="722.4" cy="1332" r="143.5" fill="none" stroke="#1e1e78" stroke-width="18.02"/><circle cx="722.4" cy="1332" r="47.84" fill="none" stroke="#1e1e78" stroke-width="15.02"/><path d="m512.1 1251a225.2 225.2 0 0147.45-74.98" fill="none" stroke="#fff" stroke-linecap="round" stroke-width="9.009"/><path transform="scale(-1)" d="m-922.3-1368a203.2 203.2 0 0173.9-122.8" fill="none" stroke="#fff" stroke-linecap="round" stroke-width="9.009"/><path transform="scale(-1)" d="m-932.7-1412a225.2 225.2 0 0147.45-74.98" fill="none" stroke="#fff" stroke-linecap="round" stroke-width="9.009"/><path d="m697.3 382.5a120.1 120.1 0 01-108.6 125.8 120.1 120.1 0 01-130.4-103 120.1 120.1 0 0197.31-134.7" fill="none" stroke="#1e1e78" stroke-linecap="round" stroke-width="30.03"/><path d="m225.7 1353a117.1 117.1 0 01-54.34-134 117.1 117.1 0 01117.2-84.68 117.1 117.1 0 01110.1 93.64 117.1 117.1 0 01-64.71 129.3" fill="none" stroke="#1e1e78" stroke-linecap="round" stroke-width="30.03"/><path d="m569.9 1636s97.15-25.89 101.3-65.67" fill="#00f" stroke="#1e1e78" stroke-linecap="round" stroke-width="30.03"/><path d="m183.3 1327a126.1 126.1 0 01-13.03-130.9" fill="none" stroke="#fff" stroke-linecap="round" stroke-width="9.009"/><path d="m149.2 1318a150.2 150.2 0 01-12.09-98.1" fill="none" stroke="#fff" stroke-linecap="round" stroke-width="9.009"/><path d="m471.9 314.1a126.1 126.1 0 01115.7-51.86" fill="none" stroke="#fff" stroke-linecap="round" stroke-width="9.009"/><path d="m477 273.3a150.2 150.2 0 0179.26-34.7" fill="none" stroke="#fff" stroke-linecap="round" stroke-width="9.009"/></g><path d="m37.64 42.54s4.665-2.643 4.15-7.437c-.5153-4.794-5.921-4.067-6.353-8.086-.4319-4.018 1.704-4.677 1.704-4.677s-5.195 1.743-3.006 7.26c1.344 3.387 3.546 4.108 5 6.331 2.171 3.319-1.494 6.609-1.494 6.609z" fill="#fff" opacity=".75" stroke-width=".07892"/><path d="m42.4 28.49s-3.584-1.302-3.584-4.254 3.972-2.865 3.972-5.339c0-2.474-1.598-2.735-1.598-2.735s3.898.7194 2.743 4.221c-.7097 2.15-2.249 2.733-3.129 4.182-1.314 2.163 1.597 3.925 1.597 3.925z" fill="#fff" opacity=".75" stroke-width=".05271"/></svg>`;

let session = null;

document.addEventListener("DOMContentLoaded", async () => {
  // core logic: send the yt link to MRC
  let sendCurPageToMRC = () => {
    let videoId = (new URL(location.href)).searchParams.get("v");
    if(!videoId) return;
    if(!session || session.readyState != WebSocket.OPEN) return;

    const msg = {
      msg: "yt",
      link: `https://www.youtube.com/watch?v=${videoId}`,
    };
    session.send(JSON.stringify(msg));
  };

  // add the button and its style
  let button = document.createElement("button");
  button.id = "send-to-makereal-cafe";
  button.innerHTML = icon;
  button.title = "Send to Makereal Cafe!";
  button.hidden = true;
  document.body.append(button);
  button.addEventListener("click", sendCurPageToMRC);

  document.body.insertAdjacentHTML("beforeend", `<style>
    #send-to-makereal-cafe {
      position: fixed;
      right: 0; top: 50vh;
      translate: 0 -50%;
      opacity: 1;

      z-index: 10000;

      display: flex;
      align-items: center;
      justify-content: center;

      width: 5em;
      height: 5em;
      border-radius: 50%;
      padding: 1em;

      border: solid 0.25em hsl(51, 100%, 68%);
      box-shadow: 0 0 1em hsl(51, 94%, 48%);

      background-color: #1e1e78;
      overflow: hidden;

      transition: translate 0.5s, opacity 0.5s, border 0.5s, box-shadow 0.5s;
      cursor: pointer;
    }

    #send-to-makereal-cafe:not(:hover):not(:focus-within) {
      transition: translate 0.5s 3s, opacity 0.5s 3s, border 0.5s, box-shadow 0.5s;
      translate: 50% -50%;
      opacity: 0;

      border: solid 0.25em hsl(51, 94%, 48%);
      box-shadow: 0 0 0 transparent;
    }
  </style>`);

  // try to build connection and show button,
  // resolve into bool indicating either connected or failed
  let connectMRC = () => new Promise(res => {
    console.log("connecting MRC")
    let tmpSession = new WebSocket("wss://pi.makereallabs.org/ws/");

    // if successfully connected, show the button
    let successConnect, failConnect;
    tmpSession.addEventListener("open", successConnect = () => {
      session = tmpSession;
      button.hidden = false;
      console.log("Connected!");
      res(true);
    }, { once: true });

    // wait till failed or closed, hide the button
    tmpSession.addEventListener("close", failConnect = () => {
      session = null;
      button.hidden = true;
      tmpSession.removeEventListener("open", successConnect);
      console.log("Disconnected!");
      res(false);
    }, { once: true });
  });
  connectMRC();

  let networkChangeCount = 0;
  let networkChangeHandler = (whatEvent, { retryCnt = 20, maxTimeout = 5 * 60 * 1000 }) => async () => {
    console.log("Network Change!");

    let timeout = 1000;
    let myNetworkChangeID = ++networkChangeCount;

    // keep check connection until another network change || retry too many times || successfully connected
    // timeout before next attempt is increased exponentially
    while((myNetworkChangeID == networkChangeCount) && retryCnt --> 0 && !(await connectMRC())) {
      // if not connected, wait for timeout or network change
      await new Promise(res => {
        const endPromise = () => {
          clearTimeout(timeoutId);

          if(whatEvent == "online")
            window.removeEventListener("offline", endPromise);
          else if(whatEvent == "networkChange")
            navigator.connection.removeEventListener("change", endPromise);

          res();
        };

        timeout = Math.min(timeout * 2, maxTimeout);
        let timeoutId = setTimeout(endPromise, timeout);

        if(whatEvent == "online")
          window.addEventListener("offline", endPromise);
        else if(whatEvent == "networkChange")
          navigator.connection.addEventListener("change", endPromise);
      });
    }
  }

  // once back online (or there's any sign), try to rebuild the connection for 20 times at most
  if(navigator.platform.match(/linux/i)) {
    // "online/offline" events doesn't work on linux
    if(navigator.connection) // chromium fallbacks to navigator.connection
      navigator.connection.addEventListener("change", networkChangeHandler("networkChange"));
    else { // firefox fallbacks to busy waiting
      while(true) {
        console.log("busy waiting cycle in 5s.");
        await new Promise(res => setTimeout(res, 5000));
        if(session) continue;
        console.log("MRC attempt.");
        await networkChangeHandler("online", { retryCnt: Infinity, maxTimeout: 5000 })();
      }
    }
  } else {
    window.addEventListener("online", networkChangeHandler("online"));
  }

  // dragable
  /*let x = 0, y = 0, mousedown = false, moving = false;

  button.addEventListener("mousedown", e => {
    mousedown = true;
  });

  document.documentElement.addEventListener("mousemove", e => {
    if(!mousedown) return;
    moving = true;
    x += e.movementX / document.documentElement.clientWidth;
    y += e.movementY / document.documentElement.clientHeight;
    console.log("x, y", x, y)
    button.style.translate = `${100 * x}vw ${100 * y}vh`;
  });

  document.body.addEventListener("mouseup", e => {
    if(mousedown && !moving) {
      sendCurPageToMRC();
    }
    mousedown = false;
    moving = false;
  });*/
});
