===description===
exitIsLikeThrow
===file===
<?php
/**
 * @param 1|2|3 $i
 */
function foo(int $i): void {
    $a = match ($i) {
        1 => exit(),
        2, 3 => $i,
    };
    $a === "aaa";
}
===expect===
DocblockTypeContradiction
===ignore===
TODO
