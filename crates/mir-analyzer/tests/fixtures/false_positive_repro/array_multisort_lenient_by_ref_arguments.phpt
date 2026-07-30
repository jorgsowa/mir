===description===
`array_multisort()` is uniquely lenient about by-ref args (verified against
real PHP — no "Only variables should be passed by reference" notice
either): the sort-key array arguments don't need to be lvalues, and the
trailing SORT_* order/flags scalars bound to its `&...$rest` stub slot
aren't actually passed by reference at all despite the stub declaring the
whole variadic tail as such.
===file===
<?php
function test(array $data): void {
    array_multisort(array_column($data, 'k'), SORT_ASC, $data);
}
===expect===
