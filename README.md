# CSS-colors
## Description
This encoder is made specifically for encoding hex strings - for SIGBOVIK or CSS hole from code.golf. It's searching for shortest in bytes solution. There's implemented multithreading, so searching should be rather fast - any suggestions are really appreciated :)
## Usage
Install rust with cargo.

You can edit main.rs file for some tweaks like:
- Changing index range 8..14
- Change hash function
- Change type of accumulation - if there should be e = hash or e += hash
- Change char range you're using default 1..0xFFFF
- cange MAX_CACHE_SIZE - default 16 * 10e5 - beware here - cache is really hungry for your RAM

Then run:
```
cargo run
```

Example output:

```
INDEX: 12
14/05/2024 21:38:19 1 of 5 - 287 Best Size: 0
14/05/2024 21:38:19 2 of 5 - 4739 Best Size: 2
14/05/2024 21:38:19 3 of 5 - 2298 Best Size: 3
14/05/2024 21:38:19 4 of 5 - 862 Best Size: 4
14/05/2024 21:38:19 5 of 5 - 4969 Best Size: 5
Encoding Complete:
INDEX: 13
14/05/2024 21:38:19 1 of 5 - 277 Best Size: 0
14/05/2024 21:38:19 2 of 5 - 4215 Best Size: 2
14/05/2024 21:38:19 3 of 5 - 4589 Best Size: 3
14/05/2024 21:38:19 4 of 5 - 2276 Best Size: 4
14/05/2024 21:38:19 5 of 5 - 7218 Best Size: 5
Encoding Complete:
Byte Count 12: 6
Byte Count 13: 7
```

That means better compression is for index 12 -> file out/rs_12.txt
There should be result code:

```js
for(w=i=e=2;e=e*9.1/`-ǉp%?*`.charCodeAt(i++/2);)w=e.toString(16)[12]+w // 0f8fffaebd2 yey! 🎉
```

# DISCLAIMER
This code is fully inspired by Luke's blog: https://www.luke-g.com/sigbovik-golf-horse/

