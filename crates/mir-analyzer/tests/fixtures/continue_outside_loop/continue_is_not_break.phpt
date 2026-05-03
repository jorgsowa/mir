===description===
continueIsNotBreak
===file===
<?php
                    switch(2) {
                        case 2:
                            echo "two
";
                            continue 2;
                    }
===expect===
ContinueOutsideLoop
===ignore===
TODO
