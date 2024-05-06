const fs = require('fs')
const INDEX = 8
const BASE_16 = "0123456789abcdef"
// The provided utility functions
  function byteSize(string) {
    return new Blob([string]).size;
  }
  function isValidChar(x) {
    return [13, 36, 92, 96].indexOf(x) === -1;
  }
  
  function hash(store, value) {
    return store / value;
  }
  
  function createHash(start, x, size) {
    let hashValues = [hash(start, x)];
    for (let i = 1; i < size; i++) {
        hashValues.push(hash(start + hashValues.reduce((a, b) => a + b, 0), x));
    }
    return hashValues;
}
  
  function checkCondition(start, hashValues, pair, y = 10) {
    for (let i = 0; i < pair.length; i++) {
        let sum = start + hashValues.slice(0, i + 1).reduce((a, b) => a + b, 0);
        if (get_hex_digit(sum, y) != pair[i]) {
            return false;
        }
    }

    return true;
}


function get_hex_digit(x, d)
{
    //return parseInt(x.toString(16) [d], 16)
    let shift = 1;
    let threshhold = 16;
    for(;x >= threshhold; ++shift) threshhold *= 16;
    if(d == shift) {
        return -1
    } else if (d < shift) {
        return BASE_16[x * 16 ** (1 - shift + d) & 15]
    } else {
        let adj_x = x* 16 ** (d - shift)
        let ret = adj_x & 15

        if (ret == 0 && adj_x%1 == 0) {
            return -1
        } else {
            return BASE_16[ret]
        }
    }
}
  
  // State and Encoder classes to implement the new logic
  class State {
    constructor({start, currentPairIndex, byteCount, history}) {
      this.start = start;
      this.currentPairIndex = currentPairIndex; // Points to the current pair in the hex string
      this.byteCount = byteCount; // Byte count used in encoding
      this.history = history; // Keep track of encoding history
    }
  }
  
  class Encoder {
    constructor(initialState, targetHex, targetHexPairs, cacheSize) {
      this.initialState = initialState;
      this.targetHex = targetHex; // Target hex string
      this.targetHexPairs = targetHexPairs; // Hex pairs to be matched
      this.cacheSize = cacheSize;
      this.oldCache = new Set([initialState]); // Initialize with the initial state
      this.newCache = new Set();

      this.possibleChars = [];
      // Generate all valid characters
      for (let char = 1; char < 0xfff; char++) {
        if (isValidChar(char)) {
            this.possibleChars.push(String.fromCharCode(char));
        }
      }
    }
  
    generateNextStates(state) {      
      const nextStates = [];
      for (const char of this.possibleChars) {
        const currentPair = this.targetHexPairs[state.currentPairIndex]; // Current hex pair
        const currentHash = createHash(state.start, char.charCodeAt(), currentPair.length);
        const conditionMet = checkCondition(
          state.start,
          currentHash,
          currentPair, 
          INDEX
        );

        if (conditionMet) {
          const newState = new State({
            start: state.start + currentHash.reduce((a, b) => a + b, 0),
            currentPairIndex: state.currentPairIndex + 1,
            byteCount: byteSize(state.history.join("") + char),
            history: [...state.history, char]
          });

          if (newState.byteCount < 650) {
            nextStates.push(newState);
          }
        }
      }
      return nextStates;
    }
  
    pruneCache() {
      if (this.newCache.size > this.cacheSize) {
        console.log('Pruning cache')
        const sortedStates = Array.from(this.newCache).sort((a, b) => a.byteCount - b.byteCount);
        this.newCache = new Set(sortedStates.slice(0, this.cacheSize));
      }
    }
  
    encode() {
      let completed = false;
      let counter = 0;
      while (!completed) {
        for (const state of this.oldCache) {
          const nextStates = this.generateNextStates(state);

        //   if (nextStates.length === 0) {
        //     console.log('No next states found')
        //     return;
        //   }
  
          nextStates.forEach((s) => this.newCache.add(s));
        }

        console.log(++counter, ' of ', this.targetHexPairs.length, 'New Cache:', this.newCache.size)
  
        this.pruneCache(); // Keep the cache within the size limit
  
        // Check if we've reached the desired outcome
        for (const state of this.newCache) {
          if (state.currentPairIndex >= this.targetHexPairs.length) {
            completed = true;
            console.log('Completed')

            this.test(state.history.join(''))
            return state; // This is the optimal solution
          }
        }
  
        this.oldCache = new Set(this.newCache);
        this.newCache.clear();
      }
  
      return null;
    }

    test(state) {
        let errors = 0
        let w=''
        let i=0
        let e=2
        const y = INDEX
        for(;e+=hash(e,state.charCodeAt(i/2));i++) {
            w+=e.toString(16)[y]
    
            if(e.toString(16)[y] === undefined) {
                // console.log(i/2|0, state.charCodeAt(i/2) , 'undefined', 'e:', e, 'w:', e.toString(16)[y], 'm:', this.targetHex[i-2])  // Debugging output
                errors++
            }
            
            if ((this.targetHex[i-2] && e.toString(16)[y]) != this.targetHex[i-2] ) {
                // console.log(i/2|0, state.charCodeAt(i/2), 'Not matching', 'e:', e, 'w:', e.toString(16)[y], 'm:', this.targetHex[i-2])  // Debugging output
                errors++
            }
        }

        console.log('Equals:', w === this.targetHex)
    }
  }
  
  // Example Usage
  const targetHex = "0f8fffae"; // Target hex string
//   const targetHex = "0f8fffaebd700ffff7fffd4f0fffff5f5dcffe4c4000000ffebcd0000ff8a2be2a52a2adeb8875f9ea07fff00d2691eff7f506495edfff8dcdc143c00ffff00008b008b8bb8860ba9a9a9006400a9a9a9bdb76b8b008b556b2fff8c009932cc8b0000e9967a8fbc8f483d8b2f4f4f2f4f4f00ced19400d3ff149300bfff6969696969691e90ffb22222fffaf0228b22ff00ffdcdcdcf8f8ffffd700daa520808080008000adff2f808080f0fff0ff69b4cd5c5c4b0082fffff0f0e68ce6e6fafff0f57cfc00fffacdadd8e6f08080e0fffffafad2d3d3d390ee90d3d3d3ffb6c1ffa07a20b2aa87cefa778899778899b0c4deffffe000ff0032cd32faf0e6ff00ff80000066cdaa0000cdba55d39370db3cb3717b68ee00fa9a48d1ccc71585191970f5fffaffe4e1ffe4b5ffdead000080fdf5e68080006b8e23ffa500ff4500da70d6eee8aa98fb98afeeeedb7093ffefd5ffdab9cd853fffc0cbdda0ddb0e0e6800080663399ff0000bc8f8f4169e18b4513fa8072f4a4602e8b57fff5eea0522dc0c0c087ceeb6a5acd708090708090fffafa00ff7f4682b4d2b48c008080d8bfd8ff634740e0d0ee82eef5deb3fffffff5f5f5ffff009acd32"; // Target hex string
  const targetHexPairs = targetHex.match(/.{1,2}/g); // Create pairs from the hex string
  console.log(targetHexPairs)

  const initialState = new State({
    start: 2, currentPairIndex: 0, byteCount: 0, history: []
  }); 
  const encoder = new Encoder(initialState, targetHex, targetHexPairs, 100000); // Cache size of 1000
  
  const result = encoder.encode();
  
  if (result) {
    console.log("Encoding Complete:");
    console.log("Encoded Sequence:", result.history.join(""));
    console.log("Byte Count:", result.byteCount);

    fs.writeFile('a.txt', `for(w='f',i=0,e=2;e+=e/\`${result.history.join("")}\`.charCodeAt(i++/2);)w+=e.toString(16)[${INDEX}]`, (err) => {
        if (err) {
            console.log(err)
        }
    })
  } else {
    console.log("No valid encoding found.");
  }
  