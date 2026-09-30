===description===
Prohibit callable with required arg
===file===
<?php
/**
 * @param Closure():int $x
 */
function accept_closure($x) : void {
    $x();
}
accept_closure(
  function (int $x) : int {
//^ +2:3 InvalidArgument: Argument $x of accept_closure() expects 'callable with 0 required parameter(s)', got 'callable with 1 required parameter(s)'
    return $x;
  }
);
===expect===
