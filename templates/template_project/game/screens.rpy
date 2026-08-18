# Simplified Screens for Novelize
screen say(who, what):
    style_prefix "say"
    window:
        id "window"
        if who is not None:
            window:
                style "say_namebox"
                text who id "who"
        text what id "what"

style say_window is default:
    xalign 0.5
    xfill True
    yalign 1.0
    yminimum 150
    background "#000000aa"
    padding (20, 20, 20, 20)

style say_namebox is default:
    xalign 0.05
    yalign 0.0
    xoffset 5
    yoffset -40
    background "#333333dd"
    padding (10, 5, 10, 5)

style say_dialogue is default:
    xalign 0.0
    yalign 0.0
    xoffset 10
    yoffset 10
    color "#ffffff"
    size 22

# Minimal main menu so we can run the template
screen main_menu():
    tag menu
    style_prefix "main_menu"
    frame:
        xalign 0.5
        yalign 0.5
        has vbox:
            spacing 20
        text "Novelize Visual Novel" size 40 xalign 0.5
        textbutton "Start Game" action Start() xalign 0.5
        textbutton "Quit" action Quit(confirm=False) xalign 0.5

# Navigation screen for other menus if needed
screen navigation():
    vbox:
        textbutton "Return" action Return()
