===description===
`+=` onto a union of shapes adds the right-hand keys to every alternative.
===file===
<?php
function describe(bool $deprecated): string {
    $map = [];
    if ($deprecated) {
        $map = ['Deprecated' => 'deprecated'];
    }
    $map += ['Default' => 'default'];
    $out = '';
    foreach ($map as $label => $name) {
        if ('default' === $name) {
            $out .= $label;
        }
    }
    return $out;
}
