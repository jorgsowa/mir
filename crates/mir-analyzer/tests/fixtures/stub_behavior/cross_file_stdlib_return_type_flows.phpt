===description===
cross file stdlib return type flows
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:Clock.php===
<?php
function now(): \DateTimeImmutable {
    return new \DateTimeImmutable();
}
===file:Main.php===
<?php
$dt = now();
$formatted = $dt->format('Y-m-d');
