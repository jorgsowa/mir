===description===
Exit is like throw
===ignore===
TODO
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
