# Simplified Screens for Novelize
screen say(who, what):
    style_prefix "say"
    window:
        id "window"
        vbox:
            spacing 10
            xfill True
            if who is not None:
                window:
                    style "say_namebox"
                    text who id "who"
            text what id "what"

style say_window:
    xalign 0.5
    yalign 0.98
    xsize 1180
    yminimum 180
    background Solid("#050505ee")
    padding (25, 20, 25, 20)

style say_namebox:
    background Solid("#882222d0")
    padding (12, 6, 12, 6)
    yminimum 35

style say_label:
    color "#ffffff"
    size 22
    bold True

style say_dialogue:
    color "#e0e0e0"
    size 22
    line_spacing 4

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
