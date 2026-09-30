===description===
MixedArrayAccess fires when accessing a mixed function return as an array.
===file===
<?php
function getMixed(): mixed {
    return [];
}
echo getMixed()[0];
//   ^^^^^^^^^^^^^ MixedArrayAccess: Array access on mixed type
===expect===
