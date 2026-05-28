===description===
Array-destructuring targets used in conditional branch not reported as unused
===file===
<?php
/** @return array{string, string, string} */
function getMatches(): array { return []; }

function parse(): void {
    // $label and $link used in conditional branch, $target used in condition
    [$label, $link, $target] = getMatches();
    if ($target !== '') {
        echo $label;
        echo $link;
    }
}
===expect===
