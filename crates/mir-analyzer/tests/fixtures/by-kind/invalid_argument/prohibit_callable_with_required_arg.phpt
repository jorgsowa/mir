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
    return $x;
  }
);
===expect===
InvalidArgument@9:2-11:3: Argument $x of accept_closure() expects 'callable with 0 required parameter(s)', got 'callable with 1 required parameter(s)'
