// Game scene
// -------------
// Runs the core gameplay loop

$(function() {
  WSocket = new WebSocket($("body").data("ws-url"));
  
        
  WSocket.onmessage = function(event) {
    var message;
    message = JSON.parse(event.data);
    switch (message.type) {
      case "loadScene":
       // console.log(message);
		Crafty.trigger("ClearScene",message);
        return Crafty.trigger("LoadScene",message);
      case "userMove":
      //	console.log(message);
		return Crafty.trigger("UserMove", message);
      case "userSubscribe":
          console.log(message);
		return Crafty.trigger("UserSubscribe", message);   
      default:
        return console.log(message);
    }
  };
  
    Game.start();
    
    Crafty.scene('Loading'); 
    setTimeout(function(){ // Subscribe to scene
    subscribe = {type: "subscribe"};
    WSocket.send(JSON.stringify(subscribe));
    }, 2000);

    
       
});


Crafty.scene('Game', function() {

    
    // Show the victory screen once all villages are visisted
	this.show_victory = this.bind('VillageVisited', function() {
		if (!Crafty('Village').length) {
			Crafty.scene('Victory');
		}
	});
    
}, function() {
	// Remove our event binding from above so that we don't
	//  end up having multiple redundant event watchers after
	//  multiple restarts of the game
	this.unbind('VillageVisited', this.show_victory);
});


// Victory scene
// -------------
// Tells the player when they've won and lets them start a new game
Crafty.scene('Victory', function() {
	// Display some text in celebration of the victory
	Crafty.e('2D, DOM, Text')
		.text('All villages visited!')
		.attr({ x: 0, y: Game.height()/2 - 24, w: Game.width() })
		.css($text_css);

	// Give'em a round of applause!
	Crafty.audio.play('applause');

	// After a short delay, watch for the player to press a key, then restart
	// the game when a key is pressed
	var delay = true;
	setTimeout(function() { delay = false; }, 5000);
	this.restart_game = Crafty.bind('KeyDown', function() {
		if (!delay) {
			Crafty.scene('Game');
		}
	});
}, function() {
	// Remove our event binding from above so that we don't
	//  end up having multiple redundant event watchers after
	//  multiple restarts of the game
	this.unbind('KeyDown', this.restart_game);
});


// Loading scene
// -------------


Crafty.scene('SceneTransition', function(){ 

    this.bind('LoadScene', function(message) {    
            // Place a tree at every edge square on our grid of 16x16 tiles
            for (var x = 0; x < Game.map_grid.width; x++) {
            	for (var y = 0; y < Game.map_grid.height; y++) {
        			switch (message.scene[x][y]) {
        			 	case 0: // Ground earth
                            //Crafty.e('Tree').at(x, y);
                            break; 
						case 1: // Ground Grass
                            //Crafty.e('Tree').at(x, y);
                            break; 
                        case 2: // Tree
                            Crafty.e('Tree').at(x, y);
                            break;
                        case 3: 
                            Crafty.e('Bush').at(x, y);
                            break;
                        case 4: 
                            Crafty.e('Rock').at(x, y);
                            break;
                        case 5: 
                            Crafty.e('Village').at(x, y);
                            break;
                        case 6: 
                            Crafty.e('Water').at(x, y);
                            break;
                        default: 
        			}
        		}
        	}

						this.player = Crafty.e('PlayerCharacter').at(0, 0);
          
          this.player.bind('EnterFrame', function () {
	
            if (this.y != this.oy || this.x != this.ox) {
    			this.oy = this.y;
    			this.ox = this.x;
                userMove = {type: "userMove", x: this.x, y: this.y};
                WSocket.send(JSON.stringify(userMove));
    			
    		}

			
			
            });
            
          this.player1 = Crafty.e('PlayerCharacter1').at(-1, -1).attr({alpha: 0.8});
			
			this.player1.bind('UserMove', function (message) {
				//alert(message);
				this.x = message.x;
				this.y = message.y;
				
				
					var data = {x: this.x-this.ox, y: this.y-this.oy };
					this.oy = this.y;
					this.ox = this.x;
					this.trigger("NewDirection", data);
				
				
				
			});
			
		 this.player1.bind('EnterFrame', function () {
	
            if (this.y != this.oy || this.x != this.ox) {
    			this.oy = this.y;
    			this.ox = this.x;
               
    			this.stop();
				} 
            });		
			
			this.player.x=0;
			this.player.y=0;
			
			// this.player1.x=-5;
			// this.player1.y=-5;
			
        
	    Crafty.viewport.follow(this.player, 0, 0);
        
	});
    
       Crafty.viewport.init(200,200);
                                          


});
    
    






// Handles the loading of binary assets such as images and audio files
Crafty.scene('Loading', function(){
	// Draw some text for the player to see in case the file
	//  takes a noticeable amount of time to load
	Crafty.e('2D, DOM, Text')
		.text('Loading; please wait...')
		.attr({ x: 0, y: Game.height()/2 - 24, w: Game.width() })
		.css($text_css);

	// Load our sprite map image
	Crafty.load([
		'assets/images/16x16_forest_2.gif',
		'assets/images/hunter.png',
		'assets/images/treasure.png',
		'assets/images/water.gif',
		'assets/images/hunter1.png',
		'assets/images/hunter2.png',
		'assets/images/hunter3.png',
		'assets/images/hunter4.png',
		'assets/images/hunter5.png',
		'assets/images/door_knock_3x.mp3',
		'assets/images/door_knock_3x.ogg',
		'assets/images/door_knock_3x.aac',
		'assets/images/board_room_applause.mp3',
		'assets/images/board_room_applause.ogg',
		'assets/images/board_room_applause.aac',
		'assets/images/candy_dish_lid.mp3',
		'assets/images/candy_dish_lid.ogg',
		'assets/images/candy_dish_lid.aac'
		], function(){
		// Once the images are loaded...

		// Define the individual sprites in the image
		// Each one (spr_tree, etc.) becomes a component
		// These components' names are prefixed with "spr_"
		//  to remind us that they simply cause the entity
		//  to be drawn with a certain sprite
		Crafty.sprite(16, 'assets/images/16x16_forest_2.gif', {
			spr_tree:    [0, 0],
			spr_bush:    [1, 0],
			spr_rock:    [1, 1]
		});
		
		Crafty.sprite(16, 'assets/images/water.gif', {
			spr_water:    [0, 0]
		});

		Crafty.sprite(16, 'assets/images/treasure.png', {
			spr_treasure:    [0, 0]
		});


		// Define the PC's sprite to be the first sprite in the third row of the
		//  animation sprite map
		Crafty.sprite(16, 'assets/images/hunter.png', {
			spr_player:  [0, 2]
		}, 0, 2);
		
	    Crafty.sprite(16, 'assets/images/hunter1.png', {
			spr_player1:  [0, 2]
		}, 0, 2);

		Crafty.sprite(16, 'assets/images/hunter2.png', {
			spr_player2:  [0, 2]
		}, 0, 2);
		
		Crafty.sprite(16, 'assets/images/hunter3.png', {
			spr_player3:  [0, 2]
		}, 0, 2);
			    
		Crafty.sprite(16, 'assets/images/hunter4.png', {
			spr_player4:  [0, 2]
		}, 0, 2);
		
		Crafty.sprite(16, 'assets/images/hunter5.png', {
			spr_player5:  [0, 2]
		}, 0, 2);
		
		// Define our sounds for later use
		Crafty.audio.add({
			knock: 	  ['assets/door_knock_3x.mp3', 'assets/door_knock_3x.ogg', 'assets/door_knock_3x.aac'],
			applause: ['assets/board_room_applause.mp3', 'assets/board_room_applause.ogg', 'assets/board_room_applause.aac'],
			ring:     ['assets/candy_dish_lid.mp3', 'assets/candy_dish_lid.ogg', 'assets/candy_dish_lid.aac']
		});

		
		// Now that our sprites are ready to draw, start the game
		Crafty.scene('SceneTransition');
		
	   
	})
});


      