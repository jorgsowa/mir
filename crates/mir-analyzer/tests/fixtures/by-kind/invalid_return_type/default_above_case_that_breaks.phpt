===description===
Default above case that breaks
===file===
<?php
function foo(string $a) : string {
//                               ^ +12:1 InvalidReturnType: Return type 'void' is not compatible with declared 'string'
  switch ($a) {
    case "a":
      return "hello";

    default:
    case "b":
      break;

    case "c":
      return "goodbye";
  }
}
===expect===
