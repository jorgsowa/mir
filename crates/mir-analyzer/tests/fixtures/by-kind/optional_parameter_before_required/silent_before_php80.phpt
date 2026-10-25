===description===
PHP 7.4 accepts an optional parameter before a required one without a notice.
===config===
<mir>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php
function pad(string $text = '', int $width): string
{
    return str_pad($text, $width);
}
