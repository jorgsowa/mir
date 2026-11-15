===description===
No seed, a seed from a secure source and a configured seed aren't reported, and
the kind is hidden unless enabled.
===config===
<mir>
  <issueHandlers>
    <PredictableSeed errorLevel="warning"/>
  </issueHandlers>
</mir>
===file===
<?php
function seed(int $configured): void {
    mt_srand();
    mt_srand(random_int(0, PHP_INT_MAX));
    mt_srand($configured);
}
