===description===
ini_get_all() returns an array with string keys; the omitted $details is a bool, so the conditional return is undecidable and the foreach value widens to mixed (MixedAssignment), while the key stays string — no PossiblyInvalidArgument on $key
===file===
<?php
foreach (ini_get_all() as $key => $_value) {
    echo strtolower($key);
}
===expect===
MixedAssignment@2:34-2:41: Variable $_value is assigned a mixed type
