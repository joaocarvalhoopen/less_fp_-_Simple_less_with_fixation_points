# less_fp - Simple less with fixation points.
In principle it's is supposed to help read a little bit faster.

## Description: 
This simple program reads a text file, paginate it and shows it with fixation points in bold. In principal they are supposed to allow you to read faster. <br>
Use Esc to quit. <br>
Use 'q' to prev_page. <br>
Use 'a' to next_page. <br>
Use mouse or keyboard for terminal resize. <br>
I tested it under Linux. <br>


## Screenshots
Help screen <br>
<br>

![Program help](./img/less_fp_help.png) <br>


Output screen <br>


![Output text example](./img/less_fp_help.png) <br>


## TODO
* Implement simple Search keys / + text_to_search + enter, with next and previous key bindings, the found words will be negative highlighted. The pages will automatically jump to the next or the previous page to go to the nearest word.


## Dependencies
```
clap = "3.1.18"
unic-normal = "0.9.0"
crossterm = "0.23.2"
```


## References
* Bionic Reading <br>
  [https://bionic-reading.com/](https://bionic-reading.com/)

* Understanding The Concept Of Eye Fixation <br>
  [https://www.speedreadinglounge.com/eye-fixation](https://www.speedreadinglounge.com/eye-fixation)


## License: 
MIT Open Source license.


## Have fun!
Best regards, <br>
João Nuno Carvalho
