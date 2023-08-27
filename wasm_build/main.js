import init, {main, } from './game_of_life_bevy.js'
console.log('X')
  const runBevyApp = async () => {
    console.log('xd')
    await init();

    main();

    /** @type {HTMLCanvasElement} */
    const canvas = document.querySelector("#my-canvas")
    // canvas.width = window.innerWidth

  };
  

  window.addEventListener('load', runBevyApp)