===description===
A function a polyfill declares behind `function_exists()` in several files is
checked with each file's own parameters and return type, not another file's.
===file:bootstrap.php===
<?php
if (!function_exists('grapheme_len')) {
    function grapheme_len(string $input): int { return strlen($input); }
}
echo grapheme_len('a');
===file:bootstrap80.php===
<?php
if (!function_exists('grapheme_len')) {
    function grapheme_len(?string $string): int { return strlen((string) $string); }
}
echo grapheme_len('a');
===file:bootstrap81.php===
<?php
if (!function_exists('grapheme_len')) {
    function grapheme_len(array $input): int { return count($input); }
}
echo grapheme_len(['a']);
===file:bootstrap85.php===
<?php
if (!function_exists('grapheme_len')) {
    function grapheme_len(?string $string, string $locale = ''): int|false {
        return strlen((string) $string) + strlen($locale) + strlen($missing);
//                                                                 ^^^^^^^^ UndefinedVariable: Variable $missing is not defined
//                                                                 ^^^^^^^^ MixedArgument: Argument $string of strlen() is mixed
    }
}
echo grapheme_len('a', 'en');
