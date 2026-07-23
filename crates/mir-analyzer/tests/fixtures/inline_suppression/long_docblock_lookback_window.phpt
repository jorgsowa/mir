===description===
A @psalm-suppress InvalidDocblock in an unusually long multi-line
docblock (60+ lines) previously fell outside the fixed 30-line
look-back window used to decide whether a named suppression is
"used" -- the collector-emitted InvalidDocblock lands at the
docblock-start line, far above the actual malformed tag, so a
genuinely-used suppression was wrongly reported as unused.
===file===
<?php

/**
 * @psalm-suppress InvalidDocblock
 * @property int $p1
 * @property int $p2
 * @property int $p3
 * @property int $p4
 * @property int $p5
 * @property int $p6
 * @property int $p7
 * @property int $p8
 * @property int $p9
 * @property int $p10
 * @property int $p11
 * @property int $p12
 * @property int $p13
 * @property int $p14
 * @property int $p15
 * @property int $p16
 * @property int $p17
 * @property int $p18
 * @property int $p19
 * @property int $p20
 * @property int $p21
 * @property int $p22
 * @property int $p23
 * @property int $p24
 * @property int $p25
 * @property int $p26
 * @property int $p27
 * @property int $p28
 * @property int $p29
 * @property int $p30
 * @property int $p31
 * @property int $p32
 * @property int $p33
 * @property int $p34
 * @property int $p35
 * @property int $p36
 * @property int $p37
 * @property int $p38
 * @property int $p39
 * @property int $p40
 * @property int $p41
 * @property int $p42
 * @property int $p43
 * @property int $p44
 * @property int $p45
 * @property int $p46
 * @property int $p47
 * @property int $p48
 * @property int $p49
 * @property int $p50
 * @param 'foo $x
 */
function bar($x): void {}
===expect===
MissingParamType@57:13-57:15: Parameter $x of bar() has no type annotation
UnusedParam@57:13-57:15: Parameter $x is never used
