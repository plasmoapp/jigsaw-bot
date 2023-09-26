console.log("Hello world!");
window.Telegram.WebApp.expand();
// document.body.appendChild(app.view);

// main();

// async function main() {
//   try {
//     const puzzleUuid = window.location.pathname.split("/")[2];
//     if (!puzzleUuid) throw new Error("Puzzle Uuid is undefined");
//     const response = await fetch(`/api/puzzle/${puzzleUuid}`);
//     const puzzleData = await response.json();


//     // const previewImg = document.createElement("img");
//     // previewImg.setAttribute("src", `/assets/${puzzleUuid}/source.jpeg`);
//     // document.body.appendChild(previewImg);

//     // for (const [key, value] of Object.entries(puzzleData)) {
//       // const tileImg = document.createElement("img");
//       // tileImg.setAttribute("src", `/assets/${puzzleUuid}/${key}.jpeg`);
//       // document.body.appendChild(tileImg);
//     // }
//   } catch(error) {
//     console.error(error);
//   }
// }