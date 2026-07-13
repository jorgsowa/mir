===description===
A genuinely too-few-arguments call against an intersection-typed callable
(no spread involved) is still flagged — the arity-only fallback isn't
disabled wholesale, only for spread/unknown-arity calls.
===file===
<?php
/**
 * @param object&Closure(int,int,int):void $fn
 */
function test(object $fn): void {
    $fn(1, 2);
}
===expect===
TooFewArguments@6:4-6:13: Too few arguments for callable(): expected 3, got 2
