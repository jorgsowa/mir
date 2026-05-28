===description===
Array-destructuring targets used in if branch that always returns/diverges not reported as unused
===file===
<?php
/** @return array{string, string, string} */
function getMatches(): array { return []; }

function parse(): string {
    [$label, $link, $target] = getMatches();
    if ($target !== '') {
        return $label . $link . $target;
    }
    return '';
}
===expect===
