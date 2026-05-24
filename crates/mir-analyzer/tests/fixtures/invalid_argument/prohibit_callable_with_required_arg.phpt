===description===
prohibitCallableWithRequiredArg
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
InvalidArgument
===ignore===
TODO
