//////////////////////////////////////////////////////////////////////
//
//  FILE:       naglatex.h
//              Translationtable for NAG values to Tex.
//
//  Copyright (c) 2000-2013 Shane Hudson and others.
//
//  Author:     Shane Hudson (sgh@users.sourceforge.net)
//  Updated:	W. van den Akker
//
//////////////////////////////////////////////////////////////////////


const char * evalNagsLatex [] =
{
	"",  // one for the offset
	"!",  // $1
	"?",  // $2 
	"!!",  // $3
	"??",  // $4
	"!?",  // $5
	"?!",  // $6
	"forced",  // $7
	" \\xskakcomment{\\onlymove}",  	// $8
	"worst",  // $9
	" \\xskakcomment{\\equal}",  		// $10
	"",  // $11
	"{$\\leftrightarrows$}",  // $12 
	" \\xskakcomment{\\unclear}",  	// $13
	" \\xskakcomment{\\wbetter}",  	// $14
	" \\xskakcomment{\\bbetter}",  	// $15
	" \\xskakcomment{\\wupperhand}",  	// $16
	" \\xskakcomment{\\bupperhand}",  	// $17
	" \\xskakcomment{\\wdecisive}",  	// $18
	" \\xskakcomment{\\bdecisive}",  	// $19
	"",  // $20
	"",  // $21
	" \\xskakcomment{\\zugzwang}",  	// $22 
	" \\xskakcomment{\\zugzwang}",  // $23
	"",  // $24
	"",  // $25
	" \\xskakcomment{\\moreroom}",  	// $26
	"",  // $27
	"",  // $28
	"",  // $29
	"{$\\circlearrowleft$}",  // $30
	"{$\\circlearrowright$}",  // $31
	"",  // $32 
	"",  // $33
	"",  // $34
	" \\xskakcomment{\\devadvantage}",	// $35
	" \\xskakcomment{\\withinit}",  	// $36
	"",  // $37
	"",  // $38
	"",  // $39
	" \\xskakcomment{\\withattack}",  	// $40
	"",  // $41
	"",  // $42 
	"",  // $43
	" \\xskakcomment{\\compensation}",  // $44
	"",  // $45
	"",  // $46
	"",  // $47
	"{$$\\boxplus$$}",  // $48
	"{$$\\boxplus$$}",  // $49
	" \\xskakcomment{\\centre}",  		// $50
	" \\xskakcomment{\\centre}",  // $51
	"",  // $52 
	"",  // $53
	"",  // $54
	"",  // $55
	"",  // $56
	"",  // $57
	" \\xskakcomment{\\kside}",  		// $58
	"",  // $59
	"",  // $60
	"",  // $61
	" \\xskakcomment{\\qside}",  		// $62 
	"",  // $63
	"",  // $64
	"",  // $65
	"",  // $66
	"",  // $67
	"",  // $68
	"",  // $69
	"",  // $70
	"",  // $71
	"",  // $72 
	"",  // $73
	"",  // $74
	"",  // $75
	"",  // $76
	"",  // $77
	"",  // $78
	"",  // $79
	"",  // $80
	"",  // $81
	"",  // $82 
	"",  // $83
	"",  // $84
	"",  // $85
	"",  // $86
	"",  // $87
	"",  // $88
	"",  // $89
	"",  // $90
	"",  // $91
	"",  // $92 
	"",  // $93
	"",  // $94
	"",  // $95
	"",  // $96
	"",  // $97
	"",  // $98
	"",  // $99
	"",  // $100
	"",  // $101
	"",  // $102 
	"",  // $103
	"",  // $104
	"",  // $105
	"",  // $106
	"",  // $107
	"",  // $108
	"",  // $109
	"",  // $110
	"",  // $111
	"",  // $112 
	"",  // $113
	"",  // $114
	"",  // $115
	"",  // $116
	"",  // $117
	"",  // $118
	"",  // $119
	"",  // $120
	"",  // $121
	"",  // $122 
	"",  // $123
	"",  // $124
	"",  // $125
	"",  // $126
	"",  // $127
	"",  // $128
	"",  // $129
	"",  // $130
	"",  // $131
	" \\xskakcomment{\\counterplay}",	// $132
	"",  // $133
	"",  // $134
	"",  // $135
	" \\xskakcomment{\\timelimt}",  	// $136
	"",  // $137
	"",  // $138
	"",  // $139
	" \\xskakcomment{\\withidea}",  	// $140
	"",  // $141
	" \\xskakcomment{\\betteris}",  	// $142 
	"",  // $143
	" \\xskakcomment{\\various}",  	// $144
	" \\xskakcomment{\\comment}",  	// $145
	" \\xskakcomment{\\novelty}",  	// $146
	" \\xskakcomment{\\weakpt}",  		// $147
	" \\xskakcomment{\\ending}",  		// $148
	" \\xskakcomment{\\file}",  		// $149
	" \\xskakcomment{\\diagonal}",  	// $150
	" \\xskakcomment{\\bishoppair}",  	// $151
	"",	  // $152
	" \\xskakcomment{\\opposbishops}",  	// $153 
	" \\xskakcomment{\\samebishops}",  	// $154
	"",  // $155
	"",  // $156
	"",  // $157
	"",  // $158
	"",  // $159
	"",  // $160
	"",  // $161
	"",  // $162 
	"",  // $163
	"",  // $164
	"",  // $165
	"",  // $166
	"",  // $167
	"",  // $168
	"",  // $169
	"",  // $170
	"",  // $171
	"",  // $172 
	"",  // $173
	"",  // $174
	"",  // $175
	"",  // $176
	"",  // $177
	"",  // $178
	"",  // $179
	"",  // $180
	"",  // $181
	"",  // $182 
	"",  // $183
	"",  // $184
	"",  // $185
	"",  // $186
	"",  // $187
	"",  // $188
	"",  // $189
	" \\xskakcomment{\\etc}",  		// $190
	" \\xskakcomment{\\doublepawns}", 	// $191
	" \\xskakcomment{\\seppawns}",  	// $192
	" \\xskakcomment{\\unitedpawns}",  	// $193
	"",  // $194
	"",  // $195
	"",  // $196
	"",  // $197
	"",  // $198
	"",  // $199
	"",  // $200
	"",  // $201
	"",  // $202
	"",  // $203
	"",  // $204
	"",  // $205
	"",  // $206
	"",  // $207
	"",  // $208
	"",  // $209
	" \\xskakcomment{\\see}",  		// $210
	" \\xskakcomment{\\mate}",  		// $211
	" \\xskakcomment{\\passedpawn}",  	// $212
	" \\xskakcomment{\\morepawns}",  	// $213
	" \\xskakcomment{\\with}",  		// $214
	" \\xskakcomment{\\without}",  	// $215
};
