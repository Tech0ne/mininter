Alive = {}
Alive.hp = 10
Alive.name = "Alive"
Alive.present = function()
    print "Hi, i'm " + self.name + " and I have " + self.hp + " hp !"
end function

Human = new Alive
Human.hp = 50
Human.name = "Human"

joe = new Human
joe.hp = 100
joe.name = "joe"

joe.present
