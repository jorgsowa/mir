===description===
MixedArrayAccess fires when accessing a mixed function return as an array.
===file===
<?php
function getMixed(): mixed {
    return [];
}
echo getMixed()[0];
===expect===
MixedArrayAccess@5:5-5:18: Array access on mixed type
