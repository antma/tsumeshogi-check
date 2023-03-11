use super::piece;
const BLACK_PIECES: [u64; 729] = [
  16630493352329319041,
  7585613607221143487,
  13385612154864935341,
  12196255603298630044,
  8654316807562249701,
  13692944393919728573,
  6844355083735842685,
  12322979096530320894,
  4392208110469915832,
  10937856472192103173,
  9401188201208402163,
  11640281496023921017,
  12153367770483297052,
  9344181372254446130,
  1843334104387655039,
  3024221277812823532,
  17387086170894840111,
  13585631200489004119,
  18355836731383238092,
  445232736631927722,
  11026332940165068816,
  5107075908415038753,
  3478516474546786643,
  17583600147654380454,
  14602811776937652997,
  15302230152275240652,
  13137272033086942446,
  7752757835250510993,
  4227617414604295454,
  14658887349810851313,
  2454579734474694801,
  14993094832184899934,
  3594783875518591360,
  14981895440838250949,
  18167056841237567277,
  9133617878997676351,
  2332181050249312721,
  13824449836353370104,
  16802045003508566149,
  8596301990579296903,
  11343999302222523025,
  9149333538566919459,
  1521580799534256535,
  1411292445546872897,
  14783784306844402313,
  8878649328665000927,
  8726725336662046346,
  13440652545796984781,
  2489827878203038437,
  10463331916177858750,
  7484560921844313306,
  6076827097501090484,
  2689401048109136167,
  1753963564126660530,
  14020007231040959623,
  16292163437563804291,
  7671260136535487345,
  17948218596120338684,
  12621388227109949184,
  15655911363783290095,
  873945899015997719,
  11765595421867607853,
  16940205611288068099,
  1795802643941291648,
  5190847610848619367,
  10719106903928937031,
  18179958250328679832,
  1368375398888150018,
  5867673022555273289,
  15117296036457089702,
  14536863727006148941,
  8148855563403100278,
  4699383206989761364,
  1027339613735678957,
  12143026988419565081,
  17100533325407204605,
  15997904881072944850,
  3300746728633118135,
  4256749143058655268,
  15923895087292137757,
  9668673759576036167,
  1128599018197411755,
  16133795921837493730,
  821288056206962005,
  6910569734251882179,
  10136383586435756203,
  8484452605961301703,
  8882863699771563525,
  11196530883601085069,
  13290686227509701664,
  2969956433816723890,
  3861487663046781215,
  13592002795707576100,
  4485424386919196596,
  570095138043312867,
  29925726993099217,
  4878765466557956539,
  9588625621233658315,
  16940490888466958538,
  9838522525843168445,
  4914168753501281185,
  16327201023436412191,
  299150103894119378,
  3775871108956475029,
  5710570438488221101,
  16555508712104566649,
  15195509150745559425,
  16453544357540973533,
  16768153056916393926,
  472706470361906655,
  13102008087447540693,
  4823726799030144250,
  16066239368708717906,
  5877178671541663378,
  12663585792153931091,
  171138406389389930,
  12992742155547905468,
  8607307929136620720,
  13692110300149847644,
  16228213718502174179,
  4515107373789772995,
  9244875680082305066,
  11501876370763718369,
  11751532081518304652,
  13279460647188507382,
  14194018183575850447,
  6154115107834614975,
  16608704957727121388,
  5700001219674528060,
  6302744569986447945,
  7561832287397528382,
  3478836375156179837,
  4678728106659867946,
  12873349355808625387,
  6971210769966737091,
  5883476434698637054,
  9327483558781342305,
  8972120088312827798,
  3155752173891198307,
  2264853167353546823,
  163681838800831934,
  4959498869120795804,
  3401122610826896312,
  3772784095421068444,
  8341781898940439297,
  14812407314542723479,
  15824080312131525943,
  4360163999160552756,
  15647203970228910816,
  12196626273623249695,
  12339544107545691359,
  9985477822848273097,
  9413004302127252705,
  5462891847843700719,
  12798573983246944421,
  15488348242585012603,
  76269773102442066,
  4794681638561516215,
  15263989823230379493,
  1922736238952143109,
  5202435498661337736,
  16097409662184032806,
  2856743833799670978,
  2462941373822064906,
  2713854099011829283,
  16859703366907113932,
  16542147666537559296,
  716640694423588914,
  4656602913595208736,
  8224510762874137924,
  1669131050993533543,
  7856747606617949739,
  1376751398412637697,
  7984645486218236688,
  1790298770569491609,
  14737895131036157112,
  2885993878575686135,
  11317905912531829693,
  9613093591147659077,
  11247584602832060038,
  7718224966318466158,
  8717676652839564405,
  12884614644120824421,
  3356439815846962536,
  15725001773827569903,
  5795906501396853756,
  6524540529910176056,
  9364904122723975057,
  17876602123935628178,
  16763069563942661989,
  5988515385272246709,
  8520096099270925042,
  14767117157552680594,
  5972375453564246955,
  7693713394343238864,
  17219853687796895171,
  17723420400195912818,
  2683399962177157440,
  15834010967254799803,
  2520504364241514529,
  16260883391806242389,
  12954692717471916607,
  7364437103411565711,
  5347490798271086257,
  1772645378786996692,
  17517463069237160579,
  8595078395860394639,
  807995651500653474,
  6884790789895289473,
  4527510961234043437,
  16767596126881450503,
  4444323444036179615,
  857408234027706080,
  3325280933673692206,
  1857906068186752082,
  17658701231859542186,
  9906714641428231046,
  495387266011391759,
  17208255266535541233,
  9370725084492951885,
  13878364857706708786,
  16654909808832583314,
  13558002479672168638,
  2459059099844034830,
  7925824315663045911,
  11547091839334474764,
  12753899607691279716,
  2325817080920026391,
  16512797174066630752,
  13340103094579103887,
  7453360419056429693,
  2777060636018607058,
  5437321034641493323,
  5141844250352432050,
  14067521455521657959,
  15745548596746203608,
  11991870114541685579,
  7049730478690764931,
  2251197328336253825,
  600568106635912297,
  13853352552115570446,
  4546297271602338454,
  1243443190342285941,
  739383752870512042,
  9422739067837730801,
  19082308201633128,
  21889694232680921,
  13037241692720921993,
  4760866793292638317,
  7709876055834606633,
  14544284674542123684,
  3317419339573093139,
  9926406040504684148,
  2760586902974656534,
  13346386438478804022,
  13540818832477585387,
  1860925870650505310,
  9264610729354714160,
  3275849870617917035,
  17394765574630812796,
  3171582061268428734,
  6106173412356671150,
  12308657131958367392,
  18017913394667311401,
  13767067935303358972,
  18062825727609947205,
  5664083405157516976,
  2437702623555131123,
  15215934761764040977,
  15247911704531478958,
  16223736366790030269,
  7290134968963866831,
  4092069959780085733,
  1512759267778303212,
  4287600434412472236,
  12809141875184989760,
  10717228794512974679,
  704626298042597840,
  2404674680505188745,
  13026524006457324168,
  4581227032183313449,
  4394614272226257553,
  10105665530376907105,
  7903346469366834767,
  11706497556130040457,
  16621829739304812001,
  3281766643387879409,
  7849168284804834078,
  394464501759788245,
  11379673048907683373,
  2146892769685741073,
  1272774408026362101,
  13183774083422509835,
  15058993339424661115,
  16428178903738126001,
  9632359754549332269,
  12918103902146601028,
  9462931052957793775,
  2763057505747090060,
  1482527167204921897,
  16206879239097563153,
  4193629802580630304,
  1276375992057943633,
  13619775025156827108,
  6806914643379612302,
  858055150650922746,
  11557785495618663930,
  13933771795128383408,
  8255519342891025235,
  5171106595370106671,
  2400229858888757846,
  18322462285130150800,
  10699730052933868132,
  15530282379930903853,
  4947451724250459069,
  18055753543739275058,
  16333189292066542692,
  6337765978125974841,
  16086111788464526808,
  11233933639514247712,
  4975103017944099841,
  4482420512747128693,
  8804983352740327839,
  3690482026803598767,
  12464620524821176552,
  2371346714533897692,
  8986828098510562142,
  2620940496977885297,
  2372771266998724055,
  14188336366918749295,
  12690338533008211946,
  12431300692533563068,
  14207393947590382198,
  2341838249603543278,
  3231074749356174673,
  12637289463631335582,
  7385215046771293652,
  6743892675875491565,
  10313197555931102028,
  9227157619413012332,
  1244299003559732724,
  9591750889390794250,
  6500871229477430673,
  6239554035273878590,
  8332050907115327873,
  8975213445724998625,
  9164292477391006657,
  6942934533117409539,
  10300686711550073945,
  18126107992174997951,
  8160449038229449859,
  14512500900218964929,
  8042159087496158617,
  17039134528302636556,
  4866506070604736462,
  14530457020278071201,
  17003307005146182594,
  16552879958070127584,
  13576662300578846194,
  11598788846876293670,
  5615803998624467912,
  2862492275297288794,
  10838461702843638685,
  13556436933913323723,
  6420669128880024328,
  6229789676656010106,
  9712650814719817387,
  13326059755907030264,
  13806484846294858590,
  1106669109338723985,
  6424454950696388448,
  2617724538169437134,
  10691386219788076147,
  18141279673560630453,
  1858427739069651808,
  5382720951609685147,
  11316972089361584435,
  1139997704145647003,
  10329524843103020150,
  6912605913860625551,
  1422578572506422223,
  12863713015943070137,
  12759376953572895117,
  15232571958890671630,
  9526566977994059898,
  12459208952871540455,
  17494263172428876686,
  13718575893952649477,
  16770196752847228321,
  250974327812588406,
  10827849021798005692,
  2890152887550519893,
  3596452110318755031,
  1467901662441946965,
  18077516896032927745,
  3186400247024082071,
  16477940593608727492,
  1258851720475594945,
  4803895285350366099,
  3814947114840101855,
  18310060297467452693,
  4936599511609685281,
  17510929809281712942,
  9721621650705104165,
  7707891785948726745,
  11323323054737450612,
  10223351211035509502,
  16408414628550473498,
  15782916703416106707,
  4385790252428964631,
  5518275483289245402,
  2169681967710233661,
  1164847566016104142,
  16045637975609287041,
  14848753058480650651,
  13189512210306513834,
  14759484714350298048,
  5587883520548427588,
  10484611597172488039,
  5341166288644385470,
  2176207989178495659,
  5356263347807772384,
  10666686515173355084,
  14488852790433274303,
  4500703919411864774,
  6653834959657619939,
  18400241228617105553,
  1544348891718418560,
  17162889206799555602,
  1687710236062538221,
  485675401038833201,
  17287172069485010532,
  15528573567924461764,
  12636717705651671074,
  14832179681036385903,
  13037971160566483782,
  17829149089701474235,
  5297979070608006629,
  9657676826238445751,
  16849844154187072658,
  7983981276573937014,
  3188336600457458265,
  14845662223642830241,
  12543970939623662155,
  17807052272617538405,
  4446124266682751965,
  414452658664098297,
  18362326549553113821,
  7562480980380570205,
  15955150122924215403,
  9616217608462726409,
  3573421074824483922,
  13652360189221744279,
  9632776535077889902,
  11913435581160554996,
  2450914273774761760,
  17301885041737696130,
  5752382427272090657,
  6223428982545720821,
  15557718419828313239,
  16184624506526989476,
  8790100746064724563,
  11781281983678739193,
  7251259400365089359,
  6398219317496927955,
  15274783204600131410,
  2346181594956007764,
  2903260903675822374,
  5079078385825603917,
  3906774120365057554,
  13262345381855881330,
  4873105172948415581,
  12454435348324112567,
  8840148185994529740,
  3782088305795605187,
  506575796879866583,
  17424936202071469775,
  6133754250590781665,
  2295682486526952535,
  11264136678001678522,
  14915899855858764758,
  17111452384938314823,
  10552158655822817977,
  7129132295730317650,
  15030137931456446876,
  14620554999213839275,
  14132979283261732240,
  17544779567479897505,
  1628959390139573714,
  5406547131758294697,
  15273102502654840422,
  4826391596859223763,
  15982951797598335159,
  4510972370271672613,
  4560639655042034368,
  3426536633377751811,
  18411700598921446397,
  10893831303519292931,
  18324565953176409000,
  13938429205576962171,
  6708245334793307130,
  9481241308193356883,
  2768591612849500684,
  2934605284683447064,
  5337912849326529699,
  10831142961930090142,
  10476585987288415409,
  1610739580834027678,
  8497213321035788076,
  12209311706979498646,
  2090918293312548454,
  9019714616268954611,
  12872386721589339117,
  12155938288004447903,
  9552461250705403052,
  5823914826216223336,
  3504176284699901839,
  1704216858105133690,
  15872335213373291443,
  14328299034561541452,
  3831110491112528855,
  13656808147910708355,
  6200666649076692803,
  7053101138989805046,
  6414994604937429970,
  1624842725834658908,
  10329206280712330324,
  6402896219354260325,
  15378057646538594978,
  748125898228310288,
  4300474607568364330,
  11361438142181872015,
  3266941935554961878,
  6792531635493329953,
  9521132580038645809,
  15547110179861382063,
  12137115831599149839,
  16698268216036925115,
  13579905067810402719,
  12281900019914300161,
  2920674877875127435,
  3646207518733360802,
  11687057212561324038,
  15529901318768626165,
  182529748429700188,
  7869271221332521803,
  9297870037662785471,
  6993905810982112430,
  12475756323252258181,
  10291044349997213063,
  16085397353771937085,
  6262883140119415628,
  17254845420323142268,
  293845202102237530,
  5800115344879365788,
  4867044231172227534,
  15880185509515029600,
  16951539209193349165,
  17323472959347462907,
  3644265122087104578,
  6141503610180471901,
  219744644670011816,
  8653852241600271759,
  3066206051450139067,
  44692780304056536,
  7366383156612384894,
  12105721264252329176,
  14322033969534255510,
  10191657045383644966,
  7870925056104392120,
  363677075927439398,
  8786294372788802297,
  18150322743152966077,
  292165326522015521,
  12573059080187404216,
  13731776362409424425,
  4877874760982251191,
  6652480428444088147,
  4299717191397371364,
  1474632404208664115,
  7267282233422951201,
  1525835552377870584,
  818925187534399395,
  3429082347855618673,
  9971434098384735404,
  10529638426568179604,
  14355194517429092476,
  3125088310352784004,
  14308729825323686977,
  14843355996052063862,
  2292318604662396069,
  5075940161020752967,
  6918113995123993579,
  16852040283370659743,
  9639743223185069838,
  13411764828811834164,
  17764732895572935058,
  15110393144695138476,
  7675405939256486823,
  13622974357618659022,
  13909273671641603207,
  14627074052083420505,
  17547821055117243132,
  5584788682280255533,
  11980835848651892955,
  2635662820578366306,
  18093696580744284005,
  8244235980834558188,
  14718733071920324557,
  10710171543571203929,
  16340472252489567727,
  1861286208092375585,
  1123664272023886864,
  620409589421392177,
  12360136253716596303,
  16192738403296169708,
  6722894215615359395,
  5631787007176228441,
  15957423446689629222,
  11336010202307879306,
  17207958286605714524,
  1185404501315583418,
  10873159304569101136,
  5386765336157321836,
  7318899141712613037,
  1088434120062830996,
  14739378117324346856,
  17420379749784985618,
  4758894451968716138,
  174268746611923321,
  16630349102567925519,
  1474630755023148524,
  9938403726233948664,
  4125546908668741383,
  14737553414998531632,
  11103967417140525076,
  14026552047187256248,
  8424827678513821923,
  9751062156798505735,
  2571005460634361262,
  12466511964194833293,
  11645718236756160666,
  10095725481012577355,
  16837479333178174524,
  5852535347766407948,
  2407035294671475142,
  17093105847659978512,
  16186392631375973711,
  6518392293663813090,
  6175557978136379057,
  10456901738002318419,
  14733463429843905434,
  13501993672282773934,
  10584152471897453457,
  8799252725881463526,
  15430143832224707694,
  2253078914391817121,
  14098845580232360232,
  6224199466033308613,
  7164514177230055038,
  3388935217386053861,
  5568096229115371022,
  18348702446785821782,
  14863288311546230714,
  6357669798337569314,
  15908462797874734390,
  10318793645363594107,
  2406616780446624484,
  12350188589077723103,
  15814151322828506537,
  10216017426616263112,
  12643924710565986131,
  16774076548579654352,
  11366703212465077386,
  9505436852620647628,
  5383143317964073436,
  9227254089865300351,
  1269823285632203649,
  6430625715498050580,
  14221943213568937659,
  15390002791972249360,
  11843212111379960819,
  12009122698118141465,
  11473411093107635129,
  279796208953920181,
  7955345031331988115,
  2828300028958710061,
  17864315156632176205,
  12922706701725107877,
  2306478902405072150,
  10960715373216162449,
  16623053974869804859,
  7630872081189456052,
  1832600030705985142,
  14680860431709124835,
  10343709702386313578,
  16835001073875875654,
  15368231887606666049,
  52020858618373454,
  8280707106841545019,
  8376583800039177071,
  15323765165003504739,
  2545757137679121435,
  8680060938667326771,
  5324449396695209964,
  7141673036437879190,
  8548851742063137047,
  11864642372050269270,
  4964614402333851676,
  13970934217971434634,
  10645367871261724860,
  11422728586063235902,
  14345227655078986527,
  14483220576860144587,
  345675842800352026,
  4952546096521277785,
  3335017885911393553,
  257433841781296475,
  9477056416308797366,
  1190102316789911560,
  10573528668324063701,
  919603354057913022,
  9748229678862928841,
  11303548216493635720,
  2456658213838697039,
  12327784113029606302,
  12683271338947470345,
  8844818751428614460,
  14589499904255762847,
];
const WHITE_PIECES: [u64; 729] = [
  11186048993999560549,
  5310250348926890499,
  17087957217053787269,
  16325048724645123201,
  13205703331958544456,
  17241453587461245740,
  16309401852546508638,
  6133920024123480681,
  6568851361019710564,
  13282912906669336007,
  4439800690153928099,
  3035221244870997831,
  9178889006152669297,
  17152180834701761787,
  157183624935374098,
  5267119779510342656,
  10472696809856537764,
  9934752199824333518,
  13875675898476158354,
  13394269525338433515,
  16412881854797996201,
  16438776624611943032,
  4504160588742208550,
  7882843958749968285,
  10119809866712358136,
  515676120134984427,
  16762814544783499675,
  3562430892901855644,
  8050727028299254332,
  5595192632853607559,
  1340802372155335767,
  11442076747043508299,
  6739500925127882659,
  8453676886990626193,
  5744460360476186719,
  2755350117055154919,
  11199041496827200943,
  12203119831425974782,
  9446695770824170952,
  1168764224012565996,
  2093085166203668558,
  547761186853147521,
  331452556951017723,
  11033319555030820637,
  4582465733077858324,
  11917159941539484746,
  17830806442180981012,
  17146397824235216201,
  7679325365696308916,
  424246950921177459,
  5889924906606020981,
  3095781189102266486,
  12619943669037558067,
  6282869323678425412,
  11145220001737900200,
  18187427331612538795,
  701224462304221625,
  16889808186427008171,
  7038321484143472303,
  2843575660598700789,
  195399124739636776,
  16936235232155415373,
  734569447185689822,
  10688875826603549054,
  7396387209475515040,
  13729904443576510491,
  7379399722001637173,
  11503308519199066652,
  16097944332170882978,
  17219059835990867820,
  3446098612108235137,
  3525255937472639468,
  741609337913308175,
  6710352738988047553,
  12762068001102974857,
  11421686953628598713,
  859574499972842096,
  2635416645966264377,
  11488537722137278261,
  8886371118211742161,
  10474214017071084768,
  9755577936011287055,
  4228773824074276307,
  13407873007087657094,
  7315501488179668804,
  7109725119180345563,
  1178946832252881980,
  9704480661732397339,
  8453768931054016751,
  7319900219650626553,
  16534555431385687797,
  2667916065486235424,
  13361778724229262910,
  6028838768113848281,
  510892104349348899,
  14749179636523669463,
  10562120636752337615,
  3116180744037659124,
  17051475521793010065,
  513695658547371188,
  13238901585997409032,
  1244098278027769766,
  5527276576102296521,
  15197346927000502961,
  3395476101699075280,
  16980385169614696926,
  17360414706540994074,
  13388784111238921734,
  5277786261430387001,
  3175634463161156303,
  7059197059429887419,
  16530795236106103674,
  15654324337324491272,
  9889440018718174578,
  9077952175642492441,
  1768744392817442402,
  10354326640721138421,
  12399950770994603715,
  15276441370681082109,
  2548660004153214316,
  18123862768999329445,
  17545601471110378729,
  15887226832469741503,
  8728318110101524499,
  204163015922742157,
  5189012201967231927,
  11011396106969783003,
  11999941936998169705,
  15880581987867845104,
  16535302348596294722,
  4798642965702045947,
  4562736183275838206,
  168899383512170281,
  8108321351724890228,
  14565118178452263883,
  2482488422522490645,
  13325337273311169778,
  5507784985420061638,
  2364381892203019246,
  2563130782887862618,
  17747950535598027538,
  4407668471554444318,
  8188895963478003256,
  59714609203251816,
  1165995317042611061,
  6911818111517286427,
  14616716721672576202,
  12056076773983755589,
  8687864808167792475,
  9227718996439085041,
  5959316568465364281,
  17285107145821000222,
  6212674994341258520,
  3365482603002779984,
  13995117882830960227,
  491360559709866272,
  5255901060601275978,
  1265809575563209828,
  7514646660272794018,
  13330994758372373397,
  3327315283661689299,
  17904908491312372589,
  13799773123980405494,
  4969512012279263508,
  1809215915888427173,
  13007646861458503699,
  12378827273962265707,
  11216724152159937472,
  3179856791179918440,
  16132722721357949796,
  4588446992404881020,
  6654559690102743531,
  15261627286869531960,
  3550640006937943130,
  214244856293540794,
  7300143956454551424,
  1340580439810097147,
  3034731412939478667,
  11019657139291587925,
  17620290264315524096,
  18122980311675026538,
  6380273796155203942,
  1829753685271922211,
  4361892612392686838,
  15214642469099770557,
  13420413275096930672,
  15917736779711143371,
  8387982806063167280,
  7933581268181184472,
  4422384683115268284,
  9614202976868726276,
  15100722251313948130,
  17995811545940502967,
  2017808133553439311,
  1942422764276251221,
  11349928247865793408,
  8497663701019541028,
  8589496767252068214,
  15229387163451232285,
  633893287278761710,
  11606731085456076694,
  11116325708487926529,
  587062874191889965,
  17180435754127375372,
  14483954097716476937,
  17666838606495071725,
  18360552231270519548,
  9618921661787925505,
  17957956982239718720,
  6785975179200953329,
  6605829212533827123,
  13388838917962941332,
  7500145389315101027,
  10525318828686957543,
  3516910793727539542,
  3704989603607887680,
  12001284186911886050,
  4808945931701370578,
  11444187974134645904,
  12678895485875738284,
  8632526369633327006,
  14107416930420906066,
  17377780405941236006,
  3444558666583787376,
  5750173833574920881,
  10158334298867308516,
  11449289323100205134,
  11223024893484635905,
  3233451457524476524,
  3923548253654613066,
  14849276809273713027,
  3993060807040383811,
  15895992093213507632,
  2258315250563786721,
  11507837359581712897,
  4121323511677291873,
  10573901439314984798,
  5255942575970975290,
  1966847452197657487,
  7856274036941627276,
  7313754812579849791,
  2316304206113868408,
  5665286570717945134,
  13818836416742406259,
  4785666104426805846,
  9412805860059900131,
  8660511474362320143,
  18228187125185341809,
  6558158362916218199,
  5412121347829517924,
  2011989877625169757,
  17426805893093788922,
  5699978662085671306,
  3991745995665141993,
  12531833182696849842,
  3991037085972347130,
  11914987140315118784,
  5614809093007167158,
  16513548555085059568,
  3550817181960207336,
  14575871212156509650,
  16263635644936470321,
  7876765818856928418,
  17972513654160101786,
  16286608469207747076,
  12004294976095872325,
  756912575316293685,
  12418028609026949121,
  16095170161914378496,
  17658155195496852626,
  5479988744423496720,
  6499131152480030056,
  16352565670197212336,
  7871239901135243924,
  83421892265413036,
  11374793715407060264,
  205044485427729438,
  934171808096781093,
  10827502172951835500,
  16259464797840335505,
  17899523543847007394,
  2814781319732924195,
  3731847191008603091,
  12795821796522757987,
  14709051305534430709,
  2088812554794763053,
  14257541667911708712,
  465592878582074452,
  2404632266335317125,
  7160152603030661595,
  14742261767335810599,
  3573429899039415057,
  14737157206597458163,
  14798858268704199796,
  2766169595743995544,
  5078558200028986198,
  4601992432029832637,
  3096792221585522803,
  15561728070899320172,
  15090314239290306165,
  11282431621322400123,
  4621819093495295171,
  2180562375966441700,
  11158348866843867465,
  14194232152199297809,
  3758336437072585818,
  10871048944795449543,
  14532820582444727233,
  13145728919401487920,
  12375403581859145150,
  1314188547425654745,
  8515487074139287424,
  10580791080632188845,
  10447493785554781699,
  14402522975487781264,
  1925577013871920256,
  8341471340907970815,
  6292309177051105974,
  6882903898078122611,
  18389914116184889697,
  12420241522514935378,
  14994646992873341685,
  3904679061378821936,
  13056648228044119876,
  16805027806750814683,
  5505333420741870941,
  9002416476731526694,
  3648956090638244569,
  14788083344216520188,
  13786675254040117989,
  3646608177123787225,
  1907664992344512359,
  7929769992118140153,
  3079749237502482197,
  321588553099103963,
  8233859545100269851,
  14630834069295497858,
  4246896595868159648,
  8810025716022299174,
  1407336571728265293,
  3784577019632076029,
  1484256542767994290,
  9421142298563661031,
  4173086196670466094,
  18328043522514426923,
  13878069500345052634,
  13681508537473143121,
  1263152769676498155,
  1286252073004801018,
  225199724976809972,
  6670248535237367783,
  18440993646907756595,
  7508046150986309166,
  2478970799487084492,
  4678051897147204362,
  5718977217226599260,
  2950327946295883729,
  10728685042511138754,
  9432635656241777722,
  1025092072302141120,
  6203600299864334066,
  7754327683488721718,
  7732917137397250526,
  11627388375864991034,
  1930540872256496995,
  15138508454467586656,
  16345505331429048513,
  11408379977938412496,
  14448993521525606576,
  9178210580969969209,
  15094801088580341701,
  17095828289944377035,
  13024218979795786010,
  6609559462009264721,
  11511257681046407682,
  10182872944139511087,
  8115492161231513772,
  7563605358198304433,
  3622508333372464202,
  5960427721057150929,
  17155662459320182561,
  13734595230394429432,
  16517958578025774011,
  12719596159449831296,
  5598953101904635643,
  15318968410151385098,
  16540028999198394838,
  18219184628062232819,
  13337614865520460850,
  4633651095612031102,
  10073018176632101915,
  4821935504176688056,
  4758215026745693480,
  8295037169638205441,
  9667219090709847684,
  9334848730364992661,
  3703553911685239689,
  3429258726603174841,
  11062889841542098540,
  11783600525767857544,
  9883909247175620065,
  3960589684333313654,
  8260592902801624005,
  13699847416036413535,
  11769330980421939585,
  5040973083604471430,
  14874347613833139156,
  8087013030429889189,
  14198979513963300372,
  12548239996432371275,
  4842991026857696880,
  15218891781991818578,
  13045181177742570317,
  15619481400810962559,
  11465330242972291275,
  2208120447262760,
  15574329244912316055,
  15828395051806221566,
  15426112434684903477,
  3730610119065202254,
  2683050533972085090,
  11289185778426503915,
  18161864633282800852,
  15972525053553183994,
  1914613246801697650,
  14383059689433727181,
  13580622269581419195,
  8067293389190706360,
  12816050248565423339,
  999007293855373405,
  14121678117414929783,
  12662901552732546778,
  1101669700474965409,
  3513056556537498689,
  722273263133476515,
  3184061607153607426,
  11015740079705545929,
  646187074559293074,
  3029815478814498620,
  13131197011528501566,
  2468260929812422515,
  15926289420314596830,
  4313147246414059484,
  16824330490282972674,
  4158880910163218325,
  2876604301902914511,
  15081773279699654482,
  16457725336401342721,
  4161606649116672087,
  3625074491981262935,
  5694257493428237989,
  662446791216107202,
  3430054984076121247,
  3034052403604648539,
  17569419922612447429,
  11573954475498906268,
  4280426436041685048,
  12038574388302696797,
  5236175031042280842,
  8784313214520577863,
  9572473369104679504,
  13942261151911790713,
  15916733198760013074,
  824091791008781616,
  285507788597872896,
  2995746201528900019,
  15316980646833694614,
  15636336229231024639,
  7896518121818602074,
  6110926381662421964,
  5093964286925234363,
  8008157379474004170,
  14249131819366253630,
  3621040930866085473,
  12086037974628189725,
  11744471232625464686,
  12988034508736758536,
  13020612673919825809,
  16343726016982726579,
  13542916721257669772,
  4906173985689114354,
  11593858762145370092,
  12821005435082135938,
  5654016300589013891,
  3386087364081707802,
  8490317802672494098,
  6154168124946140728,
  11633450109143071847,
  9564063966840703897,
  333515751905610632,
  9334013299655383030,
  17852958965925707692,
  7668634171505092570,
  14176934008357472646,
  5285060126558954068,
  12889161323709422534,
  3588262432231903517,
  13316011226742928003,
  10092233272711265527,
  12404898926682287636,
  3774028986888056086,
  733241733372758660,
  8914828348349642650,
  12827379664546252748,
  13492539334060243955,
  15692498279642956172,
  9228684678174765699,
  8140866260844376221,
  2095781588630362877,
  7904093253361589841,
  6996746655139486843,
  17524534302296297186,
  14792159490740998910,
  7292484277762106504,
  7392020608709480001,
  14134956779089088856,
  13110704743361930961,
  16691144180173039754,
  4433783601207770997,
  6733630357171625321,
  2176058265760840086,
  422048909411991344,
  10541507030503358932,
  7432790333210631085,
  14129426708321998853,
  17070621021243727121,
  5148057690342460406,
  4006024305914094466,
  4291414753729963257,
  10470416384786133186,
  6160289400676604412,
  12378840739939780853,
  4825967597412001926,
  15114120865946028032,
  9557394356904881319,
  6450439026955658679,
  18314055851629782578,
  12593976999180129499,
  13329102666736638810,
  6110684336174024156,
  5659436260845135997,
  11279455372669190534,
  10814753987942426751,
  15455296947675455417,
  18412920592933254673,
  10574277061957776681,
  11781741339040420958,
  11370168136545352287,
  5328930935111374795,
  3235543880820282024,
  10570638556045525930,
  16751622827754714194,
  4638116587358648160,
  14719806367308003685,
  11248489260805510194,
  16551002205231977760,
  1257147590870899356,
  8678456065902332489,
  8584844100037930824,
  3820105669874453911,
  13723319518055799575,
  18411136642004770253,
  17040868082252903539,
  8951401733337835137,
  3061293985532319833,
  13360595343278064455,
  17395501120528419224,
  1934491219363919881,
  6000797750394427123,
  15223273797732486196,
  260125562982257059,
  6305299257873358581,
  14563080927761806323,
  3633052693451408471,
  10301144604277168834,
  10139858666505405646,
  13664896183374248869,
  5164096004911967168,
  3110876596486833887,
  11061970606533422705,
  24876762946514003,
  15523794344983974468,
  10898415618815874371,
  3205509833320488629,
  11387960169722896257,
  16037557302366328106,
  16121908736106788980,
  2778081021478314804,
  7803692538909758451,
  7101292843428105501,
  10057313035333413115,
  8065015876360454962,
  11987759570441581463,
  3227231911971790607,
  14706811305344638422,
  5845576501714534008,
  17874765154736487224,
  1381020126368600395,
  3407293760859439,
  4340194918756774738,
  14939176338111834366,
  2745999628714035678,
  2008017272094755964,
  12578876714445492217,
  3300605126906562944,
  8239690259134710454,
  11497369478501807996,
  4842986335793279236,
  8221063638904428427,
  2720734883805355241,
  5904856644870112059,
  3293861914018449151,
  2149877273132507935,
  821552711124245527,
  1995545102510395025,
  5568232351737114612,
  11712001398378192109,
  11109539414461812971,
  2760929653605433980,
  7833098469068006289,
  18280758313533243737,
  13373321950870582474,
  11446678109673059609,
  5283107812568887575,
  401055228119384258,
  9190003150993799656,
  11564940523314525331,
  15437130195129337315,
  2492679276857889168,
  3624042489265299166,
  14116838665667890187,
  6879807375811474734,
  12300040860527914139,
  421978324290143852,
  17582805442108325760,
  2967820654069276839,
  17217611157079861844,
  6576998769317540650,
  8995973008958158690,
  4954878045649162872,
  11264211916416788236,
  9693201863901907430,
  3353000188381907580,
  17770343554003936042,
  9715353769135207149,
  1775814226338102784,
  17426133816227187411,
  12597062823122739598,
  12226865556841964646,
  3629751974860470375,
  8588864427489684554,
  8956476006905696030,
  6315685058626518397,
  12850425466807703024,
  9725840599779197345,
  3566369964882197528,
  16955687752575522535,
  5793757765408891530,
  10778489667597272845,
  4510042391107864747,
  2307995452135823729,
  740670398841751619,
  7986614993671413811,
  4803707363644241013,
  15330466512909755083,
  9241441579757203943,
  7852425940115165851,
  7840726847147363948,
  15185371678538114510,
  3239934538176772036,
  9609117979113075850,
  6489974542368561882,
  1394973835219756157,
  16679944223994527250,
  14419266131164967198,
  9948545894918666107,
  9881180991162637431,
  11626103628602783746,
  1828535379744900057,
  14724607152171411168,
  1882399698010799614,
  12920466901385045880,
  12126326877558615121,
  5321074947168730256,
  12996205522537086141,
  16485419781702620952,
  1861574715841812171,
  17315242780873928554,
  15017006160941538960,
  3915116342123278152,
  9998463243264024020,
  8029087810159375054,
  15213728949783463776,
  15604946561797381720,
  17026693089692163659,
  11987579943482025829,
  7870013560879081846,
  14786248857771315457,
  13294364545351013296,
  2874932066334771579,
  16886430585644344519,
  18168155687345073794,
  7519456052856212142,
  15459776093062169217,
  8098619801824141345,
  14499333938057783340,
  5455915225104160882,
  3927442593065476187,
  92351844164261923,
  12596881710083347916,
  12338249294912117631,
  13884167706974686419,
  15205447794137424827,
  13693952508374783004,
  14421887014323606712,
  660058929442917680,
  288713039694799970,
  7188756936977146535,
  10785558885675192013,
  13272827666846482184,
  2220545522730176585,
  14087346875347027969,
  13940667511770365607,
  10933901626925297902,
  1956344292940423610,
  8071104121496964932,
  1733385155146981565,
  2329448248514700970,
  4576464389963312381,
];
const BLACK_POCKETS: [u64; 42] = [
  5134255257892609265,
  17543561571325971529,
  6140399202515942066,
  13761077128378438653,
  16524945542898683794,
  13168527890722060257,
  9682852039492118531,
  11510465447019200843,
  6625373570519516272,
  5703441693816946585,
  8561512447974994114,
  3232230826368692764,
  14590317100608749594,
  843615298221476679,
  13318569188560459897,
  11924172408014865489,
  18023570501481013390,
  8429540417545562684,
  15685664744873296956,
  8734248926594486400,
  2756154343894101196,
  10337763810023557689,
  4347742348319088764,
  8575952667173711115,
  6039526993711559686,
  10065170364476284903,
  11566011758546397501,
  18004314883920105508,
  10641531128035986355,
  15462206273968711244,
  9516657588165214994,
  14943002518784109223,
  517029142461393488,
  1912455922067625958,
  293766640220257680,
  17867850591952344781,
  10650804662592659061,
  3493060599913633449,
  145880917807364982,
  176036850876879717,
  10387805921928811184,
  17048728356423038647,
];
const WHITE_POCKETS: [u64; 42] = [
  77835316359019554,
  18092476453905928449,
  11310620711063003941,
  4720831167258838911,
  2527981453502372497,
  457008490038085534,
  10929069362059055198,
  5570608093659970489,
  14588655847216136005,
  10917944839142755438,
  14152663850773633634,
  11312950195229600437,
  29952580427361554,
  8193641933891627781,
  16101069086985709158,
  4133550130530030738,
  2210678195277255922,
  2046878616633100534,
  15316172432529112749,
  3515609462039303378,
  9319691889774452419,
  9506395947773238505,
  9924256654861560196,
  16694078101473363509,
  1358927412457340204,
  6484390567414303432,
  14612625224245884791,
  5975609056657192955,
  16585793341460740902,
  3854934946258634829,
  11877127169380435892,
  6556938295439443217,
  16410574265502437639,
  4507931086565857790,
  15354196156793835292,
  15795295152954631373,
  5481418230542869589,
  4166864783741380382,
  18385833494679266404,
  7093169021994300255,
  17309600170858117179,
  14012059298777364097,
];

pub fn get_piece_hash(piece: i8, cell: usize) -> u64 {
  assert_ne!(piece, piece::NONE);
  let q = if piece > 0 {
    &BLACK_PIECES
  } else {
    &WHITE_PIECES
  };
  let piece = piece.abs();
  if piece >= piece::PROMOTED {
    q[cell] ^ q[81 * (piece - piece::PROMOTED) as usize + cell]
  } else {
    q[81 * piece as usize + cell]
  }
}

fn get_pocket_hash(p: &[u64], piece: i8, c: u8) -> u64 {
  assert!(piece > 0);
  if piece == piece::PAWN {
    p[23 + (c as usize)]
  } else {
    p[4 * (piece - 2) as usize + (c as usize) - 1]
  }
}

pub fn get_black_pocket_hash(piece: i8, c: u8) -> u64 {
  get_pocket_hash(&BLACK_POCKETS, piece, c)
}

pub fn get_white_pocket_hash(piece: i8, c: u8) -> u64 {
  get_pocket_hash(&WHITE_POCKETS, -piece, c)
}
