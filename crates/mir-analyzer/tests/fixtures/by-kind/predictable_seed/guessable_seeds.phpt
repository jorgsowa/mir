===description===
A literal seed, or one built from the clock or process ID, makes every later
generator result predictable.
===config===
<mir>
  <issueHandlers>
    <PredictableSeed errorLevel="warning"/>
  </issueHandlers>
</mir>
===file===
<?php
function seed(): void {
    mt_srand(1234);
//  ^^^^^^^^^^^^^^ PredictableSeed: mt_srand() seeds the generator with a guessable value, so every mt_rand() and rand() result after it is predictable
    srand(time());
//  ^^^^^^^^^^^^^ PredictableSeed: srand() seeds the generator with a guessable value, so every mt_rand() and rand() result after it is predictable
    mt_srand((int) (microtime(true) * 1000) ^ getmypid());
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PredictableSeed: mt_srand() seeds the generator with a guessable value, so every mt_rand() and rand() result after it is predictable
}
