const fs = require('fs')

const y = 8
const LIMIT = 0xfff
const m = "0f8fffae"
// const m = "0f8fffaebd700ffff7fffd4f0fffff5f5dcffe4c4000000ffebcd0000ff8a2be2a52a2adeb8875f9ea07fff00d2691eff7f506495edfff8dcdc143c00ffff00008b008b8bb8860ba9a9a9006400a9a9a9bdb76b8b008b556b2fff8c009932cc8b0000e9967a8fbc8f483d8b2f4f4f2f4f4f00ced19400d3ff149300bfff6969696969691e90ffb22222fffaf0228b22ff00ffdcdcdcf8f8ffffd700daa520808080008000adff2f808080f0fff0ff69b4cd5c5c4b0082fffff0f0e68ce6e6fafff0f57cfc00fffacdadd8e6f08080e0fffffafad2d3d3d390ee90d3d3d3ffb6c1ffa07a20b2aa87cefa778899778899b0c4deffffe000ff0032cd32faf0e6ff00ff80000066cdaa0000cdba55d39370db3cb3717b68ee00fa9a48d1ccc71585191970f5fffaffe4e1ffe4b5ffdead000080fdf5e68080006b8e23ffa500ff4500da70d6eee8aa98fb98afeeeedb7093ffefd5ffdab9cd853fffc0cbdda0ddb0e0e6800080663399ff0000bc8f8f4169e18b4513fa8072f4a4602e8b57fff5eea0522dc0c0c087ceeb6a5acd708090708090fffafa00ff7f4682b4d2b48c008080d8bfd8ff634740e0d0ee82eef5deb3fffffff5f5f5ffff009acd322"

function isValidChar(x) {
    return [13, 36, 92, 96].indexOf(x) === -1
}
// Hash function
function hash(store, value) {
    return store ** .1 * value
    // return store / value
}

// Function to get a specific digit from a hexadecimal number
function get_hex_digit(x, d) {
    return x.toString(16)[d]
}

// Function to create hash
function createHash(start, x, size) {
    let hashValues = [hash(start, x)];
    for (let i = 1; i < size; i++) {
        hashValues.push(hash(start + hashValues.reduce((a, b) => a + b, 0), x));
    }
    return hashValues;
}

// Function to check condition
function checkCondition(start, hashValues, y, pair) {
    for (let i = 0; i < pair.length; i++) {
        let sum = start + hashValues.slice(0, i + 1).reduce((a, b) => a + b, 0);
        if (get_hex_digit(sum, y) != pair[i]) {
            return false;
        }
    }

    
    return true;
}


function test(res) {
    let errors = 0
    for(w='',i=e=2;e+=hash(e,res.charCodeAt(i/2));i++) {
        w+=e.toString(16)[y]

        if(e.toString(16)[y] === undefined) {
            console.log(i/2|0, res.charCodeAt(i/2) , 'undefined', 'e:', e, 'w:', e.toString(16)[y], 'm:', m[i-2])  // Debugging output
            errors++
        }
        
        if ((m[i-2] && e.toString(16)[y]) != m[i-2] ) {
            console.log(i/2|0, res.charCodeAt(i/2), 'Not matching', 'e:', e, 'w:', e.toString(16)[y], 'm:', m[i-2])  // Debugging output
            errors++
        }
    }

    console.log('Equals:', w === m, 'Errors:', errors)

    // Write to a.txt file
    fs.writeFile('a.txt', `for(w='f',i=e=2;e+=e**.1*\`${res}\`.charCodeAt(i++/2);)w+=e.toString(16)[${y}]`, (err) => {
        if (err) {
            console.log(err)
        }
    })
}

// Main search function
const CACHE = {}


function search() {
    let res = 'f'
    let start = 2
    let counter = 1
    for (let pair of m.match(/.{1,2}/g)) {
        let x = 1
        let found = false
        while (!found && x++ < LIMIT) {
            let hashValues = createHash(start, x, pair.length)
            if (checkCondition(start, hashValues, y, pair)) {
                if (isValidChar(x)) {
                    found = true
                    res += String.fromCharCode(x)
                    start += hashValues.reduce((a, b) => a + b, 0)
                    console.log(counter++, pair, 'found', x, start)
                }
            }
            
        }

        if (!found || x == LIMIT) {
            console.log('not found :(')
            return;
        }
    }
    console.log(counter)



    test(res)
}

// Call the search function
search()