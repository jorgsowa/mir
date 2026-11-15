===description===
Decoded data used as data, plain `eval()` of program code and calls through
variables holding ordinary names aren't reported.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <PossiblyFalseArgument errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function data(string $payload, callable $handler): void {
    $image = base64_decode($payload);
    file_put_contents('/tmp/img.png', (string) $image);
    $json = gzdecode($payload);
    eval('return 1 + 1;');
    $handler($image);
    $name = 'strtoupper';
    echo $name('x');
    $sep = chr(10) . chr(13);
}
