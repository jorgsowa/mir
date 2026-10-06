===description===
User function with by-ref param reached via literal string
===file===
<?php
function fill(int &$out): void {
    $out = 1;
}
function c(): int {
    $fn = 'fill';
    $fn($n);
    return $n;
}
