/* tslint:disable */

export type Blank = 0
export type White = 1
export type Black = 2
export type Cell = Blank | White | Black
export type Board = [Cell, Cell, Cell, Cell, Cell, Cell, Cell, Cell, Cell]

/**
* @param {Board} js_objects
* @returns {Board}
*/
export function get_next_best_board(js_objects: Board): Board;
