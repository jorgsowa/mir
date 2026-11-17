===description===
A built-in function is not deprecated for targets older than the release that deprecated it.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
function describe(object $o, mixed $v): string {
    return spl_object_hash($o) . (is_double($v) ? 'f' : '') . strcoll('a', 'b') . utf8_encode('x');
//                                                                                ^^^^^^^^^^^^^^^^ DeprecatedCall: Call to deprecated function utf8_encode: 8.2 Consider to use {@link mb_convert_encoding}, {@link UConverter::transcode()} or {@link iconv()}
}
