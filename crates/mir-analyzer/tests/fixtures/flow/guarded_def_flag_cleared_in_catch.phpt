===description===
A flag the catch block sets to another value proves the variables the try body
defines wherever the flag still holds its value; not when the catch may keep it.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function catchClearsFlag(): void {
    $ok = true;
    try { $u = random_int(0, 1); } catch (Exception) { $ok = false; }
    if ($ok) { echo $u; }
}

function catchMayKeepFlag(): void {
    $ok = true;
    try { $u = random_int(0, 1); } catch (Exception) { $ok = rand(0, 1) === 1; }
    if ($ok) { echo $u; }
//                  ^^ PossiblyUndefinedVariable: Variable $u might not be defined
}
