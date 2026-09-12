===description===
Only canonical integer strings become integer array keys.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function literalKeys(): void {
    $arr = [
        '0' => 'zero',
        '1' => 'one',
        '1234' => 'many',
        '-1' => 'neg',
        '007' => 'bond',
        '+1' => 'plus',
        '-0' => 'minus-zero',
        '1.0' => 'floatish',
    ];

    /** @mir-check $arr is array{0: 'zero', 1: 'one', 1234: 'many', -1: 'neg', '007': 'bond', '+1': 'plus', '-0': 'minus-zero', '1.0': 'floatish'} */
    $_ = $arr;

    echo $arr[0];
    echo $arr[1];
    echo $arr[1234];
    echo $arr[-1];
    echo $arr['007'];
    echo $arr['+1'];
    echo $arr['-0'];
    echo $arr['1.0'];
}
===expect===
