===description===
A parameter default that the `@param` int range excludes is reported for
functions, methods and promoted constructor parameters.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param int<1,255> $v */
function fn_out(int $v = 0): void {}
//                       ^ InvalidArgument: Argument $v of fn_out() expects 'int<1, 255>', got '0'

/** @param int<0,255> $v */
function fn_ok(int $v = 0): void {}

/** @param int<1,255>|null $v */
function fn_nullable(?int $v = null): void {}

/** @param int<-5,-1> $v */
function fn_negative(int $v = -6): void {}
//                            ^^ InvalidArgument: Argument $v of fn_negative() expects 'int<-5, -1>', got '-6'

function fn_native(int $v = 0): void {}

final class Color {
    /** @param int<1,255> $level */
    public function method(int $level = 0): void {}
//                                      ^ InvalidArgument: Argument $level of Color::method() expects 'int<1, 255>', got '0'

    /** @param int<1,255> $red */
    public function __construct(private int $red = 0) {}
//                                                 ^ InvalidArgument: Argument $red of Color::__construct() expects 'int<1, 255>', got '0'

    /** @param int<1,255> $level */
    public function inRange(int $level = 128): void {
        /** @mir-check $level is int<1, 255> */
        echo $level;
    }
}
===expect===
