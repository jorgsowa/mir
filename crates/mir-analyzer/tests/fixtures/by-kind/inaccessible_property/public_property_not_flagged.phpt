===description===
InaccessibleProperty does NOT fire for public properties.
===file===
<?php
class Config
{
    public int $timeout = 30;
}

echo (new Config())->timeout;
===expect===
