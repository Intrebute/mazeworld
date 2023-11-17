# The Mazefile (.maze) file format

The mazefile format intends to support a multitude of maze types and layouts. As of writing, two maze types are specified, with room for more in the future.

The format is split into two parts, the headers, and the body. The headers specify what kind of maze is being described, along with a byte offset specifying where in the file the body begins. It also includes important data based on the maze type, such as starting and end position, bounds size, etc.

Every value described as `int` is a big-endian 32-bit unsigned integer.

## Headers

The header always begins with the four byte values `[77, 65, 90, 69]`, spelling `"MAZE"` in ASCII. Following these is the `MAZE_TYPE` value, a single byte encoding which type of maze the file describes. Following this is `BODY_START : int`, specifying at what byte index of the file the main body begins. The rest of the header fields depend on the value of `MAZE_TYPE`.

As of writing, the `MAZE_TYPE` field can has two possible valid values, `1` and `2`. `MAZE_TYPE = 1` describes a rectangular bounded maze, with possible missing cells. `MAZE_TYPE = 2` describes a circular maze. 

### MAZE_TYPE = 1

This type of maze is a rectangular maze. Its header fields are as follows:

`[WIDTH : int][HEIGHT : int][START_ROW : int][START_COL : int][END_ROW : int][END_COL : int]`

Note that the order of fields for `WIDTH` and `HEIGHT` is opposite to that of the starting and end positions. The first value, `WIDTH` describes the horizontal extent of the maze, while the first position value `START_ROW` describes the vertical offset of the position. Typically, sizes are described as `x,y` pairs while positions are described as `row,column` pairs.

### MAZE_TYPE = 2

This type of maze is a circular maze. A circular maze consists of a central cell, surrounded by concentric rings, each of which is divided into a certain number of cells. Each cell is considered adjacent to the next and previous cell in each ring. Each cell is considered adjacent to a single cell in the previous lower ring. Furthermore, each cell is considered adjacent to a certain number of cells in the next outer ring, depending on that ring's branching factor. Its header fields are as follows:

`[RING_COUNT : int][ring_profile : byte[RING_COUNT - 1]]`

`RING_COUNT` is the total number of rings in the maze. A maze with a ring count of 1 is a single central cell.

`ring_profile` is a sequence of `RING_COUNT - 1` bytes specifying how many cells in the ring above are adjacent to a single cell in the current ring. For example, a ring with `ring_profile` entry `2` means that the ring above it has twice as many cells as it does, with each cell adjacent to `2` cells "above" it.

## Body

The body contains the main bulk of data of the maze. The format of this data depends on the `MAZE_TYPE` value of the file.

### MAZE_TYPE = 1

In this type of maze, exactly `width * height` cells are laid out in row-major order. Each cell is described by a single byte as a bit mask. Each of the last four bits specifies whether or not the cell is connected to an adjacent cell in the specified direction. The masks for each direction are given in `NEWS` order. 

| Direction | Mask |
| - | - |
| North | `00001000` |
| East | `00000100` |
| West | `00000010` |
|South | `00000001` |

### MAZE_TYPE = 2

In this type of maze, the cells are described in 2 or more bytes, depending on the branching factor of the ring it is on. 