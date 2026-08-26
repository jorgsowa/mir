===description===
A conditional polyfill function must retain its defaulted offset parameter.
===file===
<?php
if (!function_exists('grapheme_extract')) {
    function grapheme_extract(?string $haystack, ?int $size, ?int $offset = 0): string|false {
        return helper((int) $offset);
    }
}

function helper(int $offset): string { return (string) $offset; }
===expect===
