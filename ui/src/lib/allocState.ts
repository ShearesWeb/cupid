// allocState.ts — filter/sort state for the Allocations screen.
import type { PositionType } from "./types.ts";

export type View = "position" | "applicant";
export type TypeFilter = "all" | PositionType;
export type PosFilter = "all" | "unfilled" | "oversub" | "prealloc";
export type AppFilter = "all" | "unallocated" | "quota" | "displaced";
export type AppSort = "name" | "choices" | "unalloc";

/** Everything the allocations screen filters on; lives in the shell so it
 *  survives a trip to a detail page and back. */
export interface AllocState {
  view: View;
  search: string;
  page: number;
  typeFilter: TypeFilter;
  posFilter: PosFilter;
  appFilter: AppFilter;
  appSort: AppSort;
}

export const initialAllocState: AllocState = {
  view: "position",
  search: "",
  page: 0,
  typeFilter: "all",
  posFilter: "all",
  appFilter: "all",
  appSort: "name",
};
