===description===
`instanceof A || instanceof B` on a generic subject keeps the disjunct that is unrelated to it
===file===
<?php
/** @template T */
interface Box {}
/**
 * @template T
 * @extends Box<T>
 */
interface SubBox extends Box {}
interface Other {}

/** @param Box<int> $b */
function f(Box $b): void {
    if ($b instanceof SubBox || $b instanceof Other) {
        /** @mir-check $b is SubBox<int>|Box<int>&Other */
        echo 1;
    }
}
